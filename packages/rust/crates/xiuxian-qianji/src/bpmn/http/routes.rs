use super::activity_evidence::{
    QianjiBpmnWorkflowCompletionActivityEvidenceInput,
    QianjiBpmnWorkflowFailureActivityEvidenceInput, matching_pending_work_for_completion,
    matching_pending_work_for_failure, now_unix_ms, record_completion_activity_evidence,
    record_failure_activity_evidence,
};
use super::bpmn_source_admission::admit_control_bpmn_source;
use super::control_trace::record_bpmn_control_trace;
use super::error_api::QianjiBpmnWorkflowHttpError;
use super::execution_graph::QianjiControlExecutionGraphHttpResponse;
use super::llm_host_work_schedule::record_bpmn_llm_host_work_schedules;
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
use super::request_api::QianjiControlOpenAiCompatibleLlmWorkerRunHttpRequest;
use super::request_api::{
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowStartHttpRequest,
    QianjiBpmnWorkflowStatusHttpQuery, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskCompleteBatchHttpRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskFailHttpRequest, QianjiBpmnWorkflowTaskFailureHttpPayload,
    QianjiBpmnWorkflowTaskReleaseHttpRequest, QianjiControlRecoveryApplyHttpRequest,
};
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
use super::response_api::QianjiControlOpenAiCompatibleLlmWorkerCompleteHttpResponse;
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
use super::response_api::QianjiControlOpenAiCompatibleLlmWorkerRunHttpResponse;
use super::response_api::{
    QianjiBpmnWorkflowCancelHttpResponse, QianjiBpmnWorkflowRunHttpResponse,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowTaskClaimHttpResponse,
    QianjiBpmnWorkflowTaskReleaseHttpResponse, QianjiControlBpmnSourceHttpResponse,
    QianjiControlDiagnosticsHttpResponse, QianjiControlHistoryHttpResponse,
    QianjiControlRecoveryApplyHttpResponse, QianjiControlRecoveryHttpResponse,
    QianjiControlRunStreamHttpResponse, QianjiControlRunSummaryHttpResponse,
};
use super::state::QianjiBpmnWorkflowHttpState;
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
use crate::bpmn::control::{
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowTaskCompletionKind,
};
use crate::bpmn::control::{
    QianjiBpmnWorkflowStatusRequest, QianjiBpmnWorkflowTaskCompleteReport,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionPayload,
};
use crate::bpmn::identity::QianjiBpmnWorkflowInstanceId;
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
use crate::qianji_server::llm_worker::{
    QianjiServerOpenAiCompatibleLlmBpmnCompletionCandidate,
    QianjiServerOpenAiCompatibleLlmWorkerLoopRequest,
    run_qianji_server_openai_compatible_llm_worker_loop,
};
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
use crate::runtime_config::{
    QianjiRuntimeLlmConfig, resolve_qianji_runtime_llm_config,
    resolve_qianji_runtime_llm_config_with_env,
};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
use std::io;
use xiuxian_qianji_bpmn_engine::BpmnHostBridge;
use xiuxian_qianji_control::{
    ControlEventKind, ControlEventRecord, ControlLedger, HotStateStore, RecoveryAttempt,
    RecoveryLoopApplicationRequest, RecoveryPolicy, RunId, apply_recovery_plan,
};

fn record_bpmn_control_projection<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    session: &crate::bpmn::session::QianjiBpmnSession,
    bpmn_source: Option<&std::path::Path>,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    record_bpmn_control_trace(
        state.activity_evidence_ledger.as_deref(),
        session,
        bpmn_source,
    )?;
    if state.runtime_env.is_none() {
        return Ok(());
    }
    record_bpmn_llm_host_work_schedules(
        state.activity_evidence_ledger.as_deref(),
        state.runtime_env.as_ref(),
        session,
        bpmn_source,
    )
}

pub(super) fn router<H>(state: QianjiBpmnWorkflowHttpState<H>) -> Router
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let router = Router::new()
        .route("/workflows/start", post(start_workflow::<H>))
        .route("/workflows/{instance_id}", get(load_workflow_status::<H>))
        .route(
            "/workflows/{instance_id}/resume",
            post(resume_workflow::<H>),
        )
        .route(
            "/workflows/{instance_id}/cancel",
            post(cancel_workflow::<H>),
        )
        .route(
            "/workflows/{instance_id}/events/poll",
            post(poll_workflow_events::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/complete",
            post(complete_workflow_task::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/complete-batch",
            post(complete_workflow_tasks_batch::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/fail",
            post(fail_workflow_task::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/claim",
            post(claim_workflow_task::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/release",
            post(release_workflow_task::<H>),
        )
        .route(
            "/control/runs/{run_id}/history",
            get(load_control_history::<H>),
        )
        .route(
            "/control/runs/{run_id}/stream",
            get(load_control_run_stream::<H>),
        )
        .route(
            "/control/runs/{run_id}/bpmn-source",
            get(load_control_bpmn_source::<H>),
        )
        .route(
            "/control/runs/{run_id}/execution-graph",
            get(load_control_execution_graph::<H>),
        )
        .route(
            "/control/runs/{run_id}/summary",
            get(load_control_summary::<H>),
        )
        .route(
            "/control/runs/{run_id}/recovery",
            get(load_control_recovery::<H>),
        )
        .route(
            "/control/runs/{run_id}/diagnostics",
            get(load_control_diagnostics::<H>),
        )
        .route(
            "/control/runs/{run_id}/recovery/apply",
            post(apply_control_recovery::<H>),
        )
        .route(
            "/control/bpmn-source/admit",
            post(admit_control_bpmn_source::<H>),
        );
    #[cfg(any(
        all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
        test
    ))]
    let router = router
        .route(
            "/control/runs/{run_id}/workers/openai-compatible-llm/run",
            post(run_control_openai_compatible_llm_worker::<H>),
        )
        .route(
            "/control/runs/{run_id}/workers/openai-compatible-llm/run-and-complete",
            post(run_control_openai_compatible_llm_worker_and_complete::<H>),
        );
    router.with_state(state)
}

async fn start_workflow<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Json(request): Json<QianjiBpmnWorkflowStartHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let request = request.into_control_request();
    let prepared = state.service.prepare_start_workflow(&request)?;
    let report = state
        .service
        .start_prepared_workflow_until_host_boundary(prepared, &state.host, false, |_, _| {})
        .await?;
    record_bpmn_control_projection(
        &state,
        &report.execution.session,
        Some(report.resolved_bpmn_path.as_path()),
    )?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn load_control_history<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlHistoryHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let events = ledger
        .load_events(&run_id)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlHistoryHttpResponse::new(
        run_id.as_str().to_owned(),
        events,
    )))
}

async fn load_control_run_stream<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlRunStreamHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let events = ledger
        .load_events(&run_id)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlRunStreamHttpResponse::from_events(
        &run_id, &events,
    )))
}

async fn load_control_bpmn_source<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlBpmnSourceHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let events = ledger
        .load_events(&run_id)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    let source_ref = control_bpmn_source_ref(&events).ok_or_else(|| {
        QianjiBpmnWorkflowHttpError::not_found(
            "bpmn_source_ref_missing",
            "control run does not include a qianji-server BPMN source reference",
        )
    })?;
    let bpmn_xml = std::fs::read_to_string(source_ref.as_str()).map_err(|error| {
        QianjiBpmnWorkflowHttpError::internal_server_error(format!(
            "failed to read BPMN source '{source_ref}': {error}"
        ))
    })?;
    Ok(Json(QianjiControlBpmnSourceHttpResponse::new(
        run_id.as_str().to_owned(),
        source_ref,
        bpmn_xml,
    )))
}

async fn load_control_execution_graph<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlExecutionGraphHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let events = ledger
        .load_events(&run_id)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlExecutionGraphHttpResponse::from_events(
        &run_id, &events,
    )))
}

async fn load_control_summary<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlRunSummaryHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let summary = ledger
        .load_operator_summary(&run_id, now_unix_ms())
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlRunSummaryHttpResponse::new(summary)))
}

async fn load_control_recovery<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlRecoveryHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let recovery = ledger
        .load_recovery_snapshot(&run_id, now_unix_ms())
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlRecoveryHttpResponse::new(recovery)))
}

async fn load_control_diagnostics<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlDiagnosticsHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let diagnostics = ledger
        .load_operator_diagnostics(&run_id, now_unix_ms())
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlDiagnosticsHttpResponse::new(diagnostics)))
}

async fn apply_control_recovery<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
    Json(request): Json<QianjiControlRecoveryApplyHttpRequest>,
) -> Result<Json<QianjiControlRecoveryApplyHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let hot_state = recovery_hot_state(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let occurred_at_ms = request.occurred_at_ms;
    let priority = request.priority;
    let attempt = recovery_attempt_from_request(request)?;
    let plan = ledger
        .load_recovery_plan(&run_id, occurred_at_ms)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    let application = apply_recovery_plan(
        ledger,
        hot_state,
        RecoveryLoopApplicationRequest::new(plan, attempt, occurred_at_ms, priority),
    )
    .await
    .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    let diagnostics = ledger
        .load_operator_diagnostics(&run_id, occurred_at_ms)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlRecoveryApplyHttpResponse::new(
        application,
        diagnostics,
    )))
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
async fn run_control_openai_compatible_llm_worker<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
    Json(request): Json<QianjiControlOpenAiCompatibleLlmWorkerRunHttpRequest>,
) -> Result<Json<QianjiControlOpenAiCompatibleLlmWorkerRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let hot_state = recovery_hot_state(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let llm_config = resolve_http_runtime_llm_config(&state)?;
    let output = run_qianji_server_openai_compatible_llm_worker_loop(
        ledger,
        hot_state,
        QianjiServerOpenAiCompatibleLlmWorkerLoopRequest {
            run_id: &run_id,
            worker_id: request.worker_id.as_str(),
            task_queue: request.task_queue.as_deref(),
            now_ms: request.now_ms,
            now_step_ms: request.now_step_ms,
            lease_ttl_ms: request.lease_ttl_ms,
            heartbeat_ttl_ms: request.heartbeat_ttl_ms,
            poll_limit: request.poll_limit,
            empty_limit: request.empty_limit,
            worker_count: request.worker_count,
            settled_at_ms: request.settled_at_ms,
            settled_step_ms: request.settled_step_ms,
            openai_compatible_base_url: llm_config.base_url.as_str(),
            openai_compatible_api_key: Some(llm_config.api_key.as_str()),
            openai_compatible_timeout_ms: request.openai_compatible_timeout_ms,
            output_artifact_dir: request.output_artifact_dir.as_path(),
            output_artifact_kind: request.output_artifact_kind.as_deref(),
        },
    )
    .await
    .map_err(|error| map_llm_worker_error(&error))?;
    Ok(Json(
        QianjiControlOpenAiCompatibleLlmWorkerRunHttpResponse::new(
            run_id.as_str().to_owned(),
            output,
        ),
    ))
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
async fn run_control_openai_compatible_llm_worker_and_complete<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
    Json(request): Json<QianjiControlOpenAiCompatibleLlmWorkerRunHttpRequest>,
) -> Result<
    Json<QianjiControlOpenAiCompatibleLlmWorkerCompleteHttpResponse>,
    QianjiBpmnWorkflowHttpError,
>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let hot_state = recovery_hot_state(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let llm_config = resolve_http_runtime_llm_config(&state)?;
    let mut remaining_polls = request.poll_limit;
    let mut completed_count = 0;
    let mut worker_runs = Vec::new();
    let mut final_workflow = None;

    while remaining_polls > 0 {
        let run_index = u32::try_from(worker_runs.len()).map_err(|error| {
            QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string())
        })?;
        let output = run_qianji_server_openai_compatible_llm_worker_loop(
            ledger,
            hot_state,
            QianjiServerOpenAiCompatibleLlmWorkerLoopRequest {
                run_id: &run_id,
                worker_id: request.worker_id.as_str(),
                task_queue: request.task_queue.as_deref(),
                now_ms: stepped_worker_ms(request.now_ms, request.now_step_ms, run_index)?,
                now_step_ms: request.now_step_ms,
                lease_ttl_ms: request.lease_ttl_ms,
                heartbeat_ttl_ms: request.heartbeat_ttl_ms,
                poll_limit: remaining_polls,
                empty_limit: request.empty_limit,
                worker_count: request.worker_count,
                settled_at_ms: stepped_worker_ms(
                    request.settled_at_ms,
                    request.settled_step_ms,
                    run_index,
                )?,
                settled_step_ms: request.settled_step_ms,
                openai_compatible_base_url: llm_config.base_url.as_str(),
                openai_compatible_api_key: Some(llm_config.api_key.as_str()),
                openai_compatible_timeout_ms: request.openai_compatible_timeout_ms,
                output_artifact_dir: request.output_artifact_dir.as_path(),
                output_artifact_kind: request.output_artifact_kind.as_deref(),
            },
        )
        .await
        .map_err(|error| map_llm_worker_error(&error))?;
        remaining_polls = remaining_polls.saturating_sub(output.processed);
        let candidates = output
            .iterations
            .iter()
            .filter_map(|iteration| iteration.bpmn_completion.as_ref())
            .cloned()
            .collect::<Vec<_>>();
        let processed = output.processed;
        worker_runs.push(output);
        if candidates.is_empty() {
            break;
        }
        for candidate in candidates {
            let complete_request = bpmn_completion_request_from_llm_candidate(&candidate)?;
            let report = complete_task_request(&state, complete_request).await?;
            final_workflow = Some(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
                &report,
            ));
            completed_count += 1;
        }
        if processed == 0 {
            break;
        }
    }

    Ok(Json(
        QianjiControlOpenAiCompatibleLlmWorkerCompleteHttpResponse::new(
            run_id.as_str().to_owned(),
            worker_runs,
            completed_count,
            final_workflow,
        ),
    ))
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
fn bpmn_completion_request_from_llm_candidate(
    candidate: &QianjiServerOpenAiCompatibleLlmBpmnCompletionCandidate,
) -> Result<QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowHttpError> {
    Ok(QianjiBpmnWorkflowTaskCompleteRequest {
        bpmn_path: std::path::PathBuf::from(candidate.bpmn_source_ref.as_str()),
        dmn_paths: Vec::new(),
        instance_id: candidate.instance_id.clone().into(),
        checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey,
        completion: QianjiBpmnWorkflowTaskCompletionPayload {
            token_id: candidate.token_id,
            process_id: candidate.process_id.clone().into(),
            activity_id: candidate.activity_id.clone().into(),
            kind: llm_candidate_completion_kind(candidate.completion_kind.as_str())?,
            data: llm_candidate_completion_data(candidate)?,
            claimant: None,
        },
        continue_until_human_boundary: false,
    })
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
fn llm_candidate_completion_kind(
    kind: &str,
) -> Result<QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowHttpError> {
    match kind {
        "task" => Ok(QianjiBpmnWorkflowTaskCompletionKind::Task),
        "send" => Ok(QianjiBpmnWorkflowTaskCompletionKind::Send),
        "service" => Ok(QianjiBpmnWorkflowTaskCompletionKind::Service),
        "script" => Ok(QianjiBpmnWorkflowTaskCompletionKind::Script),
        "user" => Ok(QianjiBpmnWorkflowTaskCompletionKind::User),
        "manual" => Ok(QianjiBpmnWorkflowTaskCompletionKind::Manual),
        other => Err(QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_llm_bpmn_completion_kind",
            format!("unsupported BPMN LLM completion kind `{other}`"),
        )),
    }
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
fn llm_candidate_completion_data(
    candidate: &QianjiServerOpenAiCompatibleLlmBpmnCompletionCandidate,
) -> Result<serde_json::Value, QianjiBpmnWorkflowHttpError> {
    let artifact_path = candidate
        .output_ref_uri
        .strip_prefix("file://")
        .unwrap_or(candidate.output_ref_uri.as_str());
    let artifact_text = std::fs::read_to_string(artifact_path).map_err(|error| {
        QianjiBpmnWorkflowHttpError::internal_server_error(format!(
            "failed to read BPMN LLM output artifact `{artifact_path}`: {error}"
        ))
    })?;
    let artifact: serde_json::Value = serde_json::from_str(&artifact_text).map_err(|error| {
        QianjiBpmnWorkflowHttpError::internal_server_error(format!(
            "BPMN LLM output artifact is not JSON: {error}"
        ))
    })?;
    let content = artifact
        .get("content")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            QianjiBpmnWorkflowHttpError::bad_request(
                "empty_llm_bpmn_completion",
                "BPMN LLM output artifact did not include non-empty content",
            )
        })?;
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(value) => Ok(shape_llm_completion_for_bpmn_outputs(
            value,
            content,
            &candidate.output_bindings,
        )),
        Err(_) => Ok(shape_llm_text_for_bpmn_outputs(
            content,
            &candidate.output_bindings,
        )),
    }
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
fn shape_llm_completion_for_bpmn_outputs(
    value: serde_json::Value,
    raw_content: &str,
    output_bindings: &[String],
) -> serde_json::Value {
    if output_bindings.is_empty() {
        return value;
    }
    if let serde_json::Value::Object(map) = &value
        && output_bindings
            .iter()
            .all(|name| map.contains_key(name.as_str()))
    {
        return value;
    }
    if output_bindings.len() == 1 {
        let output_name = &output_bindings[0];
        if let Some(content) = value.get("content").and_then(serde_json::Value::as_str) {
            return serde_json::json!({ output_name: content });
        }
        return serde_json::json!({ output_name: raw_content });
    }
    value
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
fn shape_llm_text_for_bpmn_outputs(content: &str, output_bindings: &[String]) -> serde_json::Value {
    if output_bindings.len() == 1 {
        let output_name = &output_bindings[0];
        return serde_json::json!({ output_name: content });
    }
    serde_json::json!({ "content": content })
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
fn stepped_worker_ms(
    base_ms: u64,
    step_ms: u64,
    index: u32,
) -> Result<u64, QianjiBpmnWorkflowHttpError> {
    let offset = step_ms.checked_mul(u64::from(index)).ok_or_else(|| {
        QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_openai_compatible_llm_worker_request",
            "worker loop timestamp overflow",
        )
    })?;
    base_ms.checked_add(offset).ok_or_else(|| {
        QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_openai_compatible_llm_worker_request",
            "worker loop timestamp overflow",
        )
    })
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
fn resolve_http_runtime_llm_config<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
) -> Result<QianjiRuntimeLlmConfig, QianjiBpmnWorkflowHttpError> {
    let resolved = match state.runtime_env.as_ref() {
        Some(runtime_env) => resolve_qianji_runtime_llm_config_with_env(runtime_env),
        None => resolve_qianji_runtime_llm_config(),
    };
    resolved.map_err(|error| map_runtime_llm_config_error(&error))
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
fn map_runtime_llm_config_error(error: &io::Error) -> QianjiBpmnWorkflowHttpError {
    match error.kind() {
        io::ErrorKind::NotFound | io::ErrorKind::InvalidData | io::ErrorKind::InvalidInput => {
            QianjiBpmnWorkflowHttpError::service_unavailable(
                "qianji_llm_config_unavailable",
                error.to_string(),
            )
        }
        _ => QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()),
    }
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
fn map_llm_worker_error(error: &io::Error) -> QianjiBpmnWorkflowHttpError {
    match error.kind() {
        io::ErrorKind::InvalidInput => QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_openai_compatible_llm_worker_request",
            error.to_string(),
        ),
        _ => QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()),
    }
}

fn control_bpmn_source_ref(events: &[ControlEventRecord]) -> Option<String> {
    events.iter().find_map(|record| {
        let ControlEventKind::RunCreated { metadata, .. } = &record.event.kind else {
            return None;
        };
        metadata
            .get("bpmnSourceRef")
            .or_else(|| metadata.get("bpmn_source_ref"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    })
}

fn control_ledger<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
) -> Result<&dyn ControlLedger, QianjiBpmnWorkflowHttpError> {
    state.activity_evidence_ledger.as_deref().ok_or_else(|| {
        QianjiBpmnWorkflowHttpError::service_unavailable(
            "control_ledger_unavailable",
            "qianji-server was not started with a control ledger",
        )
    })
}

fn recovery_hot_state<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
) -> Result<&dyn HotStateStore, QianjiBpmnWorkflowHttpError> {
    state.recovery_hot_state.as_deref().ok_or_else(|| {
        QianjiBpmnWorkflowHttpError::service_unavailable(
            "control_hot_state_unavailable",
            "qianji-server was not started with a recovery hot-state store",
        )
    })
}

fn parse_control_run_id(run_id: String) -> Result<RunId, QianjiBpmnWorkflowHttpError> {
    RunId::new(run_id).map_err(|_| {
        QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_control_run_id",
            "run_id must be a non-empty string",
        )
    })
}

fn recovery_attempt_from_request(
    request: QianjiControlRecoveryApplyHttpRequest,
) -> Result<RecoveryAttempt, QianjiBpmnWorkflowHttpError> {
    if request.reason.trim().is_empty() {
        return Err(QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_recovery_reason",
            "reason must be a non-empty string",
        ));
    }
    if request.max_attempts == 0 {
        return Err(QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_recovery_policy",
            "max_attempts must be greater than zero",
        ));
    }
    Ok(RecoveryAttempt {
        attempt: request.attempt,
        reason: request.reason,
        policy: RecoveryPolicy {
            max_attempts: request.max_attempts,
            backoff_ms: request.backoff_ms,
            require_human_approval: request.require_human_approval,
        },
    })
}

async fn resume_workflow<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowActionHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let request = request.into_resume_request(instance_id);
    let prepared = state.service.prepare_resume_workflow(&request).await?;
    let report = state
        .service
        .resume_prepared_workflow_until_host_boundary(prepared, &state.host, false, |_, _| {})
        .await?;
    record_bpmn_control_projection(
        &state,
        &report.execution.session,
        Some(report.resolved_bpmn_path.as_path()),
    )?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn poll_workflow_events<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowActionHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .poll_workflow_events(&request.into_event_poll_request(instance_id), &state.host)
        .await?;
    record_bpmn_control_projection(
        &state,
        &report.execution.session,
        Some(report.resolved_bpmn_path.as_path()),
    )?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn complete_workflow_task<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskCompleteHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let request = request.into_task_complete_request(instance_id);
    let report = complete_task_request(&state, request).await?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn complete_workflow_tasks_batch<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskCompleteBatchHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let request = request.into_task_complete_batch_request(instance_id)?;
    let completions = request.completions.clone();
    let resume_request = request.workflow_resume_request();
    let prepared = state
        .service
        .prepare_resume_workflow(&resume_request)
        .await?;
    let pending_work = completions
        .iter()
        .map(|completion| matching_pending_work_for_completion(&prepared, completion))
        .collect::<Vec<_>>();
    let report = state
        .service
        .complete_prepared_workflow_task_batch_until_host_boundary(prepared, &request, &state.host)
        .await?;
    record_bpmn_control_projection(
        &state,
        &report.execution.session,
        Some(report.resolved_bpmn_path.as_path()),
    )?;
    record_batch_activity_evidence(
        &state,
        &request.bpmn_path,
        &request.instance_id,
        pending_work,
        &completions,
    )?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn fail_workflow_task<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskFailHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowStatusHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let instance_id = QianjiBpmnWorkflowInstanceId::from(instance_id);
    let resume_request = request.workflow_resume_request(instance_id.clone());
    let prepared = state
        .service
        .prepare_resume_workflow(&resume_request)
        .await?;
    let pending_work =
        matching_pending_work_for_failure(&prepared, &request.failure).ok_or_else(|| {
            QianjiBpmnWorkflowHttpError::bad_request(
                "task_failure_not_pending",
                "failure identity does not match pending host work",
            )
        })?;
    record_failure_activity_evidence_for_pending(
        &state,
        &request.bpmn_path,
        &instance_id,
        &pending_work,
        &request.failure,
    )?;
    let report = state
        .service
        .load_workflow_status(&QianjiBpmnWorkflowStatusRequest {
            instance_id,
            checkpoint_backend: resume_request.checkpoint_backend,
        })
        .await?;
    Ok(Json(QianjiBpmnWorkflowStatusHttpResponse::from_report(
        &report,
    )))
}

async fn complete_task_request<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    request: QianjiBpmnWorkflowTaskCompleteRequest,
) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let resume_request = request.workflow_resume_request();
    let prepared = state
        .service
        .prepare_resume_workflow(&resume_request)
        .await?;
    let pending_work = matching_pending_work_for_completion(&prepared, &request.completion);
    let completion = request.completion.clone();
    let bpmn_path = request.bpmn_path.clone();
    let instance_id = request.instance_id.clone();
    let report = state
        .service
        .complete_prepared_workflow_task_until_host_boundary(prepared, &request, &state.host)
        .await?;
    record_bpmn_control_projection(
        state,
        &report.execution.session,
        Some(report.resolved_bpmn_path.as_path()),
    )?;
    record_single_activity_evidence(state, &bpmn_path, &instance_id, pending_work, &completion)?;
    Ok(report)
}

fn record_failure_activity_evidence_for_pending<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    bpmn_path: &std::path::Path,
    instance_id: &QianjiBpmnWorkflowInstanceId,
    pending_work: &xiuxian_qianji_bpmn_engine::PendingHostWork,
    failure: &QianjiBpmnWorkflowTaskFailureHttpPayload,
) -> Result<(), QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let Some(ledger) = state.activity_evidence_ledger.as_deref() else {
        return Ok(());
    };
    record_failure_activity_evidence(
        ledger,
        QianjiBpmnWorkflowFailureActivityEvidenceInput {
            instance_id,
            bpmn_path,
            pending_work,
            failure,
            now_ms: now_unix_ms(),
        },
    )
}

fn record_single_activity_evidence<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    bpmn_path: &std::path::Path,
    instance_id: &QianjiBpmnWorkflowInstanceId,
    pending_work: Option<xiuxian_qianji_bpmn_engine::PendingHostWork>,
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> Result<(), QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    record_batch_activity_evidence(
        state,
        bpmn_path,
        instance_id,
        vec![pending_work],
        std::slice::from_ref(completion),
    )
}

fn record_batch_activity_evidence<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    bpmn_path: &std::path::Path,
    instance_id: &QianjiBpmnWorkflowInstanceId,
    pending_work: Vec<Option<xiuxian_qianji_bpmn_engine::PendingHostWork>>,
    completions: &[QianjiBpmnWorkflowTaskCompletionPayload],
) -> Result<(), QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let Some(ledger) = state.activity_evidence_ledger.as_deref() else {
        return Ok(());
    };
    let now_ms = now_unix_ms();
    for (index, (pending_work, completion)) in
        pending_work.into_iter().zip(completions.iter()).enumerate()
    {
        let Some(pending_work) = pending_work else {
            continue;
        };
        let offset = u64::try_from(index).unwrap_or(u64::MAX).saturating_mul(3);
        record_completion_activity_evidence(
            ledger,
            QianjiBpmnWorkflowCompletionActivityEvidenceInput {
                instance_id,
                bpmn_path,
                pending_work: &pending_work,
                completion,
                now_ms: now_ms.saturating_add(offset),
            },
        )?;
    }
    Ok(())
}

async fn claim_workflow_task<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskClaimHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowTaskClaimHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .claim_workflow_task(&request.into_task_claim_request(instance_id))
        .await?;
    Ok(Json(QianjiBpmnWorkflowTaskClaimHttpResponse::from_report(
        &report,
    )))
}

async fn release_workflow_task<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskReleaseHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowTaskReleaseHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .release_workflow_task(&request.into_task_release_request(instance_id))
        .await?;
    Ok(Json(
        QianjiBpmnWorkflowTaskReleaseHttpResponse::from_report(&report),
    ))
}

async fn load_workflow_status<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Query(query): Query<QianjiBpmnWorkflowStatusHttpQuery>,
) -> Result<Json<QianjiBpmnWorkflowStatusHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .load_workflow_status(&query.into_status_request(instance_id)?)
        .await?;
    Ok(Json(QianjiBpmnWorkflowStatusHttpResponse::from_report(
        &report,
    )))
}

async fn cancel_workflow<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowActionHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowCancelHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .cancel_workflow(&request.into_cancel_request(instance_id))
        .await?;
    Ok(Json(QianjiBpmnWorkflowCancelHttpResponse::from_report(
        &report,
    )))
}
