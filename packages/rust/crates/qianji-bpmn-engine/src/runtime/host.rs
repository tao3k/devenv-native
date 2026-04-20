//! Runtime helpers for host-boundary request materialization.

use super::{
    BpmnInstanceState, PendingHostWork, parallel_multi_instance_iteration_variables,
    sequential_multi_instance_iteration_variables,
};
use crate::dmn_model_api::DmnEvaluationRequest;
use crate::error::{BpmnEngineError, Result};
use crate::host_types_api::{
    BusinessRuleTaskRequest, ManualTaskRequest, ParallelMultiInstanceContext,
    PendingHostWorkRequest, RepeatExecutionContext, SequentialMultiInstanceContext,
    ServiceTaskRequest, UserTaskRequest,
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
    let (variables, repeat) =
        sequential_multi_instance_iteration_variables(instance, node_index, &instance.variables)?
            .map_or_else(
                || {
                    parallel_multi_instance_iteration_variables(
                        instance,
                        node_index,
                        token_id,
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
            )?
            .unwrap_or_else(|| (instance.variables.clone(), None));

    Ok(match pending.kind {
        super::PendingHostWorkKind::Service => {
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id,
                token_id,
                node_index,
                variables,
                repeat,
            })
        }
        super::PendingHostWorkKind::User => PendingHostWorkRequest::User(UserTaskRequest {
            instance_id,
            token_id,
            node_index,
            variables,
            repeat,
        }),
        super::PendingHostWorkKind::Manual => PendingHostWorkRequest::Manual(ManualTaskRequest {
            instance_id,
            token_id,
            node_index,
            variables,
            repeat,
        }),
        super::PendingHostWorkKind::BusinessRule => {
            let decision = pending.decision.clone().ok_or_else(|| {
                BpmnEngineError::MissingBusinessRuleDecisionRef {
                    process_id: instance.process.process_id.to_string(),
                    node_id: pending.node_index.to_string(),
                }
            })?;
            PendingHostWorkRequest::BusinessRule(BusinessRuleTaskRequest {
                instance_id,
                token_id,
                node_index,
                evaluation: DmnEvaluationRequest::new(decision, variables),
                repeat,
            })
        }
    })
}
