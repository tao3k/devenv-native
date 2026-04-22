use super::finalize::finalize_transaction_scope_completion;
use super::queue::{
    queue_transaction_compensation_targets, route_after_intermediate_throw_compensation,
};
use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnPackage, BpmnProcessSpec, Result,
};
use crate::runtime::lifecycle::state;
use crate::runtime_instance_api::TransactionCompensationCompletionMode;

pub(crate) fn throw_compensation_end_event(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    target_activity_bpmn_id: Option<&str>,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(
        instance,
        current_node_index,
        crate::NodeRuntimeStatus::Completed,
    );
    let _ = state::remove_active_token(instance, current_token_index);

    let queued = if let Some(target_activity_bpmn_id) = target_activity_bpmn_id {
        let target_activity_index = process
            .nodes
            .iter()
            .find(|node| node.bpmn_id.as_ref() == target_activity_bpmn_id)
            .map(|node| node.index)
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "throw_compensation_end_event_missing_target_activity",
            })?;
        queue_transaction_compensation_targets(
            process,
            instance,
            std::iter::once(target_activity_index),
            now_ms,
            TransactionCompensationCompletionMode::ScopeCompletion,
        )?
    } else {
        let completed_activity_node_indices = completed_activity_node_indices(instance);
        queue_transaction_compensation_targets(
            process,
            instance,
            completed_activity_node_indices.into_iter().rev(),
            now_ms,
            TransactionCompensationCompletionMode::ScopeCompletion,
        )?
    };

    if queued {
        return Ok(());
    }

    finalize_transaction_scope_completion(package, instance, now_ms)
}

pub(crate) fn throw_compensation_intermediate_event(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    target_activity_bpmn_id: Option<&str>,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(
        instance,
        current_node_index,
        crate::NodeRuntimeStatus::Completed,
    );
    let _ = state::remove_active_token(instance, current_token_index);

    let queued = if let Some(target_activity_bpmn_id) = target_activity_bpmn_id {
        let target_activity_index = process
            .nodes
            .iter()
            .find(|node| node.bpmn_id.as_ref() == target_activity_bpmn_id)
            .map(|node| node.index)
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "throw_compensation_intermediate_event_missing_target_activity",
            })?;
        queue_transaction_compensation_targets(
            process,
            instance,
            std::iter::once(target_activity_index),
            now_ms,
            TransactionCompensationCompletionMode::IntermediateRouting {
                node_index: current_node_index,
            },
        )?
    } else {
        let completed_activity_node_indices = completed_activity_node_indices(instance);
        queue_transaction_compensation_targets(
            process,
            instance,
            completed_activity_node_indices.into_iter().rev(),
            now_ms,
            TransactionCompensationCompletionMode::IntermediateRouting {
                node_index: current_node_index,
            },
        )?
    };

    if queued {
        return Ok(());
    }

    if target_activity_bpmn_id.is_none() {
        return route_after_intermediate_throw_compensation(
            process,
            instance,
            current_token_index,
            current_node_index,
            now_ms,
        );
    }

    Err(BpmnEngineError::UnsupportedOperation {
        operation: "throw_compensation_intermediate_event_missing_handler_queue",
    })
}

fn completed_activity_node_indices(instance: &BpmnInstanceState) -> Vec<BpmnNodeIndex> {
    instance
        .call_stack
        .last()
        .and_then(|frame| frame.transaction_compensation.as_ref())
        .map(|state| state.completed_activity_node_indices.clone())
        .unwrap_or_default()
}
