//! Runtime helpers for host-boundary request materialization.

use super::{
    BpmnInstanceState, PendingHostWork, parallel_multi_instance_iteration_variables,
    sequential_multi_instance_iteration_variables,
};
use crate::dmn_model_api::DmnEvaluationRequest;
use crate::error::{BpmnEngineError, Result};
use crate::host_types_api::{
    BusinessRuleTaskRequest, ManualTaskRequest, ParallelMultiInstanceContext,
    PendingHostWorkRequest, RepeatExecutionContext, ScriptTaskRequest, SendTaskRequest,
    SequentialMultiInstanceContext, ServiceTaskRequest, UserTaskRequest,
};

/// Builds a typed host-dispatch request from the currently blocked BPMN
/// instance state.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingPendingHostWork`] when the instance is not
/// currently blocked on host work, or
/// [`BpmnEngineError::AmbiguousPendingHostWork`] when more than one pending
/// host-work entry exists.
pub(crate) fn build_pending_host_work_request_impl(
    instance: &BpmnInstanceState,
) -> Result<PendingHostWorkRequest> {
    let requests = build_pending_host_work_requests_impl(instance)?;
    match requests.as_slice() {
        [] => Err(BpmnEngineError::MissingPendingHostWork {
            instance_id: instance.instance_id.to_string(),
        }),
        [request] => Ok(request.clone()),
        requests => Err(BpmnEngineError::AmbiguousPendingHostWork {
            instance_id: instance.instance_id.to_string(),
            count: requests.len(),
        }),
    }
}

/// Builds typed host-dispatch requests from every currently blocked BPMN token.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingPendingHostWork`] when the instance is not
/// currently blocked on host work.
pub(crate) fn build_pending_host_work_requests_impl(
    instance: &BpmnInstanceState,
) -> Result<Vec<PendingHostWorkRequest>> {
    if instance.pending_host_work.is_empty() {
        return Err(BpmnEngineError::MissingPendingHostWork {
            instance_id: instance.instance_id.to_string(),
        });
    }

    instance
        .pending_host_work
        .iter()
        .map(|pending| build_pending_host_work_request_for_entry(instance, pending))
        .collect()
}

fn build_pending_host_work_request_for_entry(
    instance: &BpmnInstanceState,
    pending: &PendingHostWork,
) -> Result<PendingHostWorkRequest> {
    let instance_id = instance.instance_id.to_string();
    let token_id = pending.token_id;
    let node_index = pending.node_index;
    let process_id = pending
        .process_id
        .as_deref()
        .unwrap_or(instance.process.process_id.as_ref())
        .to_string();
    let activity_id = pending
        .activity_id
        .clone()
        .unwrap_or_else(|| format!("node#{node_index}"));
    let (variables, repeat) = resolve_pending_host_work_execution_context(instance, pending)?;

    Ok(match pending.kind {
        super::PendingHostWorkKind::Send => PendingHostWorkRequest::Send(SendTaskRequest {
            instance_id,
            token_id,
            node_index,
            message_reference: pending.event_reference.clone().ok_or(
                BpmnEngineError::UnsupportedOperation {
                    operation: "build_pending_send_task_request_missing_message_reference",
                },
            )?,
            message_name: pending.event_name.clone(),
            variables,
        }),
        super::PendingHostWorkKind::Service => {
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id,
                token_id,
                node_index,
                variables,
                repeat,
            })
        }
        super::PendingHostWorkKind::Script => PendingHostWorkRequest::Script(ScriptTaskRequest {
            instance_id,
            token_id,
            node_index,
            script_format: pending.script_format.clone(),
            script_body: pending.script_body.clone(),
            variables,
            repeat,
        }),
        super::PendingHostWorkKind::User => PendingHostWorkRequest::User(UserTaskRequest {
            instance_id,
            process_id,
            token_id,
            node_index,
            activity_id,
            variables,
            repeat,
        }),
        super::PendingHostWorkKind::Manual => PendingHostWorkRequest::Manual(ManualTaskRequest {
            instance_id,
            process_id,
            token_id,
            node_index,
            activity_id,
            variables,
            repeat,
        }),
        super::PendingHostWorkKind::BusinessRule => build_business_rule_task_request(
            instance,
            pending,
            instance_id,
            token_id,
            node_index,
            variables,
            repeat,
        )?,
    })
}

fn resolve_pending_host_work_execution_context(
    instance: &BpmnInstanceState,
    pending: &PendingHostWork,
) -> Result<(serde_json::Value, Option<RepeatExecutionContext>)> {
    let uses_active_process_context = pending
        .process_id
        .as_deref()
        .is_none_or(|process_id| process_id == instance.process.process_id.as_ref());
    if !uses_active_process_context {
        return Ok((instance.variables.clone(), None));
    }

    sequential_multi_instance_iteration_variables(
        instance,
        pending.node_index,
        &instance.variables,
    )?
    .map_or_else(
        || {
            parallel_multi_instance_iteration_variables(
                instance,
                pending.node_index,
                pending.token_id,
                &instance.variables,
            )
            .map(|repeat| {
                repeat.map(|(iteration_index, total_iterations, variables)| {
                    (
                        variables,
                        Some(RepeatExecutionContext::ParallelMultiInstance(
                            ParallelMultiInstanceContext {
                                iteration_index,
                                total_iterations,
                            },
                        )),
                    )
                })
            })
        },
        |(iteration_index, total_iterations, variables)| {
            Ok(Some((
                variables,
                Some(RepeatExecutionContext::SequentialMultiInstance(
                    SequentialMultiInstanceContext {
                        iteration_index,
                        total_iterations,
                    },
                )),
            )))
        },
    )
    .map(|context| context.unwrap_or_else(|| (instance.variables.clone(), None)))
}

fn build_business_rule_task_request(
    instance: &BpmnInstanceState,
    pending: &PendingHostWork,
    instance_id: String,
    token_id: u64,
    node_index: u32,
    variables: serde_json::Value,
    repeat: Option<RepeatExecutionContext>,
) -> Result<PendingHostWorkRequest> {
    let decision = pending.decision.clone().ok_or_else(|| {
        BpmnEngineError::MissingBusinessRuleDecisionRef {
            process_id: pending
                .process_id
                .clone()
                .unwrap_or_else(|| instance.process.process_id.to_string()),
            node_id: pending.node_index.to_string(),
        }
    })?;
    Ok(PendingHostWorkRequest::BusinessRule(
        BusinessRuleTaskRequest {
            instance_id,
            token_id,
            node_index,
            evaluation: DmnEvaluationRequest::new(decision, variables),
            repeat,
        },
    ))
}
