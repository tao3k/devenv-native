//! Route BPMN pending host-work into admitted LLM activity contracts.

use super::types::{
    BPMN_HOST_WORK_LLM_ACTIVITY_ROUTE_SCHEMA, BpmnHostWorkLlmActivityRouteInput,
    BpmnHostWorkLlmEndpointDecision, BpmnHostWorkLlmRouteDecision,
};
use crate::bpmn::{QianjiBpmnActivityId, QianjiBpmnProcessId};
use crate::workflow_config::{
    QianjiWorkflowLlmTaskConfig, QianjiWorkflowLlmTaskRetryConfig, QianjiWorkflowLlmTaskRouteConfig,
};
use std::io;
use xiuxian_qianji_control::{
    ActivityId, ActivityRetryPolicy, ActivityTask, ActivityType, ErrorCode, IdempotencyKey,
    LlmActivityRequest, LlmActivityTask, LlmModelId, TaskQueue,
};

const DEFAULT_ACTIVITY_TYPE: &str = "llm.plan";
const DEFAULT_TASK_QUEUE: &str = "llm.deepseek";
const DEFAULT_PROVIDER: &str = "openai-compatible";
const DEFAULT_IDEMPOTENCY_PREFIX: &str = "qianji:bpmn:host-work:llm";

/// Builds a Qianji-owned LLM route decision for one BPMN pending host-work item.
///
/// # Errors
///
/// Returns [`io::Error`] when the pending host-work lacks stable BPMN identity,
/// when workflow config contains invalid control-plane identifiers, or when the
/// resulting LLM activity contract fails validation.
pub fn build_bpmn_host_work_llm_activity_route(
    input: BpmnHostWorkLlmActivityRouteInput<'_>,
) -> io::Result<BpmnHostWorkLlmRouteDecision> {
    let process_id = required_process_id(input.pending_work)?;
    let activity_id = required_activity_id(input.pending_work)?;
    let endpoint = endpoint_decision(input.workflow_config, input.runtime_llm);
    let llm_activity = llm_activity_task(
        input,
        process_id.as_str(),
        activity_id.as_str(),
        endpoint.model.as_str(),
    )?;

    Ok(BpmnHostWorkLlmRouteDecision {
        schema: BPMN_HOST_WORK_LLM_ACTIVITY_ROUTE_SCHEMA.to_owned(),
        profile: input.profile.to_owned(),
        instance_id: input.instance_id.to_owned(),
        process_id,
        token_id: input.pending_work.token_id,
        node_index: input.pending_work.node_index,
        node_id: input.pending_work.node_id.clone(),
        activity_id,
        work_id: input.pending_work.work_id.clone(),
        bpmn_source_ref: input.bpmn_source_ref.map(ToOwned::to_owned),
        endpoint,
        llm_activity,
    })
}

fn endpoint_decision(
    workflow_config: &QianjiWorkflowLlmTaskConfig,
    runtime_llm: &crate::runtime_config::QianjiRuntimeLlmConfig,
) -> BpmnHostWorkLlmEndpointDecision {
    BpmnHostWorkLlmEndpointDecision {
        provider: non_empty(workflow_config.llm.provider.as_deref())
            .unwrap_or_else(|| DEFAULT_PROVIDER.to_owned()),
        model: non_empty(workflow_config.llm.model.as_deref())
            .unwrap_or_else(|| runtime_llm.model.clone()),
        base_url: non_empty(workflow_config.llm.base_url.as_deref())
            .unwrap_or_else(|| runtime_llm.base_url.clone()),
        api_key_env: non_empty(workflow_config.llm.api_key_env.as_deref())
            .unwrap_or_else(|| runtime_llm.api_key_env.clone()),
        wire_api: non_empty(workflow_config.llm.wire_api.as_deref())
            .unwrap_or_else(|| runtime_llm.wire_api.clone()),
    }
}

fn llm_activity_task(
    input: BpmnHostWorkLlmActivityRouteInput<'_>,
    process_id: &str,
    bpmn_activity_id: &str,
    model: &str,
) -> io::Result<LlmActivityTask> {
    let route = &input.workflow_config.task;
    let activity_type = control_id(
        "activity_type",
        route
            .activity_type
            .as_deref()
            .unwrap_or(DEFAULT_ACTIVITY_TYPE),
        ActivityType::new,
    )?;
    let task_queue = control_id(
        "task_queue",
        route.task_queue.as_deref().unwrap_or(DEFAULT_TASK_QUEUE),
        TaskQueue::new,
    )?;
    let control_activity_id = control_id(
        "activity_id",
        llm_activity_id(
            input.instance_id,
            bpmn_activity_id,
            input.pending_work.token_id,
        )
        .as_str(),
        ActivityId::new,
    )?;
    let idempotency_key = control_id(
        "idempotency_key",
        llm_idempotency_key(input, process_id, bpmn_activity_id).as_str(),
        IdempotencyKey::new,
    )?;
    let mut task = ActivityTask::new(
        control_activity_id,
        activity_type,
        task_queue,
        idempotency_key,
    )
    .with_input_ref(input.prompt_ref.clone());
    if let Some(timeout_ms) = route.timeout_ms {
        task = task.with_timeout_ms(timeout_ms);
    }
    if let Some(retry_policy) = retry_policy(route)? {
        task = task.with_retry_policy(retry_policy);
    }

    task.metadata = task_metadata(input, process_id, bpmn_activity_id);

    let model_id = control_id("llm_model_id", model, LlmModelId::new)?;
    let mut request = LlmActivityRequest::new(model_id, input.prompt_ref.clone());
    if let Some(context_ref) = input.context_ref {
        request = request.with_context_ref(context_ref.clone());
    }
    if let Some(response_schema_ref) = input.response_schema_ref {
        request = request.with_response_schema_ref(response_schema_ref.clone());
    }
    if let Some(temperature_millis) = route.temperature_millis {
        request = request.with_temperature_millis(temperature_millis);
    }
    if let Some(max_tokens) = route.max_tokens {
        request = request.with_max_tokens(max_tokens);
    }
    request.metadata = task.metadata.clone();

    let activity = LlmActivityTask::new(task, request);
    activity
        .validate()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    Ok(activity)
}

fn required_process_id(
    work: &crate::bpmn::QianjiBpmnPendingHostWorkHttpResponse,
) -> io::Result<String> {
    let process_id = work
        .process_id
        .as_ref()
        .map(QianjiBpmnProcessId::as_str)
        .unwrap_or_default()
        .trim();
    if process_id.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "BPMN LLM activity routing requires pending host-work process_id",
        ));
    }
    Ok(process_id.to_owned())
}

fn required_activity_id(
    work: &crate::bpmn::QianjiBpmnPendingHostWorkHttpResponse,
) -> io::Result<String> {
    let activity_id = work
        .activity_id
        .as_ref()
        .map(QianjiBpmnActivityId::as_str)
        .unwrap_or_default()
        .trim();
    if activity_id.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "BPMN LLM activity routing requires pending host-work activity_id",
        ));
    }
    Ok(activity_id.to_owned())
}

fn retry_policy(
    route: &QianjiWorkflowLlmTaskRouteConfig,
) -> io::Result<Option<ActivityRetryPolicy>> {
    let Some(config) = route.retry.as_ref() else {
        return Ok(None);
    };
    let Some(max_attempts) = config.max_attempts else {
        return Ok(None);
    };
    let mut policy = ActivityRetryPolicy::new(max_attempts)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    if let Some(initial_interval_ms) = config.initial_interval_ms {
        policy = policy.with_initial_interval_ms(initial_interval_ms);
    }
    if let Some(max_interval_ms) = config.max_interval_ms {
        policy = policy.with_max_interval_ms(max_interval_ms);
    }
    if let Some(backoff_multiplier_millis) = config.backoff_multiplier_millis {
        policy = policy
            .with_backoff_multiplier_millis(backoff_multiplier_millis)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    }
    for error_code in non_retryable_error_codes(config)? {
        policy = policy.with_non_retryable_error_code(error_code);
    }
    policy
        .validate()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    Ok(Some(policy))
}

fn non_retryable_error_codes(
    config: &QianjiWorkflowLlmTaskRetryConfig,
) -> io::Result<Vec<ErrorCode>> {
    config
        .non_retryable_error_codes
        .iter()
        .map(|code| control_id("error_code", code.as_str(), ErrorCode::new))
        .collect()
}

fn task_metadata(
    input: BpmnHostWorkLlmActivityRouteInput<'_>,
    process_id: &str,
    bpmn_activity_id: &str,
) -> serde_json::Value {
    let output_bindings = input
        .pending_work
        .output_bindings
        .iter()
        .map(|binding| binding.name.as_ref().to_owned())
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema": "qianji.bpmn.host_work.llm_activity_metadata.v1",
        "profile": input.profile,
        "instance_id": input.instance_id,
        "process_id": process_id,
        "token_id": input.pending_work.token_id,
        "node_index": input.pending_work.node_index,
        "node_id": input.pending_work.node_id.as_deref(),
        "activity_id": bpmn_activity_id,
        "work_id": input.pending_work.work_id.as_deref(),
        "bpmn_source_ref": input.bpmn_source_ref,
        "pending_work_kind": format!("{:?}", input.pending_work.kind),
        "output_bindings": output_bindings,
    })
}

fn llm_activity_id(instance_id: &str, bpmn_activity_id: &str, token_id: u64) -> String {
    format!(
        "bpmn-llm-{}-{}-{token_id}",
        sanitize_id_fragment(instance_id),
        sanitize_id_fragment(bpmn_activity_id),
    )
}

fn llm_idempotency_key(
    input: BpmnHostWorkLlmActivityRouteInput<'_>,
    process_id: &str,
    bpmn_activity_id: &str,
) -> String {
    let prefix = input
        .workflow_config
        .task
        .idempotency_key_prefix
        .as_deref()
        .and_then(|value| non_empty(Some(value)))
        .unwrap_or_else(|| DEFAULT_IDEMPOTENCY_PREFIX.to_owned());
    format!(
        "{prefix}:{}:{}:{}:{}",
        sanitize_id_fragment(input.instance_id),
        sanitize_id_fragment(process_id),
        sanitize_id_fragment(bpmn_activity_id),
        input.pending_work.token_id,
    )
}

fn control_id<T, F>(field: &'static str, value: &str, new: F) -> io::Result<T>
where
    F: FnOnce(String) -> xiuxian_qianji_control::ControlResult<T>,
{
    let value = non_empty(Some(value)).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("BPMN LLM activity routing requires non-empty {field}"),
        )
    })?;
    new(value).map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(ToOwned::to_owned)
}

fn sanitize_id_fragment(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    if out.is_empty() {
        "unknown".to_owned()
    } else {
        out
    }
}
