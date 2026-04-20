//! Runtime helpers for host-boundary request materialization.

use super::{BpmnInstanceState, PendingHostWork, sequential_multi_instance_progress};
use crate::dmn::DmnEvaluationRequest;
use crate::error::{BpmnEngineError, Result};
use crate::host::{
    BusinessRuleTaskRequest, ManualTaskRequest, PendingHostWorkRequest, RepeatExecutionContext,
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
pub fn build_pending_host_work_request(
    instance: &BpmnInstanceState,
) -> Result<PendingHostWorkRequest> {
    let requests = build_pending_host_work_requests(instance)?;
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
pub fn build_pending_host_work_requests(
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
    let variables = instance.variables.clone();
    let repeat = sequential_multi_instance_progress(instance, node_index).map(
        |(completed_iterations, total_iterations)| {
            RepeatExecutionContext::SequentialMultiInstance(SequentialMultiInstanceContext {
                iteration_index: completed_iterations,
                total_iterations,
            })
        },
    );

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
