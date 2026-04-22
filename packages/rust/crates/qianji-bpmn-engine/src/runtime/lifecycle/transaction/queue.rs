use super::finalize::{finalize_transaction_cancel_shell, finalize_transaction_scope_completion};
use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnPackage, BpmnProcessSpec,
    InstanceLifecycle, NodeRuntimeStatus, Result,
};
use crate::runtime::lifecycle::state;
use crate::runtime_instance_api::TransactionCompensationCompletionMode;

pub(crate) fn transaction_compensation_is_running(instance: &BpmnInstanceState) -> bool {
    instance
        .call_stack
        .last()
        .and_then(|frame| frame.transaction_compensation.as_ref())
        .is_some_and(|state| state.cancelling)
}

pub(crate) fn complete_compensation_handler(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    if !process.nodes[current_node_index as usize].is_for_compensation
        || !transaction_compensation_is_running(instance)
    {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "complete_compensation_handler_without_active_queue",
        });
    }

    state::set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    let next_handler = instance
        .call_stack
        .last_mut()
        .and_then(|frame| frame.transaction_compensation.as_mut())
        .and_then(|state| state.pending_handler_node_indices.pop());

    if let Some(next_handler) = next_handler {
        let _ = state::remove_active_token(instance, current_token_index);
        state::set_node_status(instance, next_handler, NodeRuntimeStatus::Queued);
        let _ = state::push_active_token_with_arrival(instance, None, next_handler);
        state::record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(());
    }

    let completion_mode = instance
        .call_stack
        .last()
        .and_then(|frame| frame.transaction_compensation.as_ref())
        .map(|state| state.completion_mode)
        .unwrap_or_default();
    match completion_mode {
        TransactionCompensationCompletionMode::CancelBoundary => {
            let _ = state::remove_active_token(instance, current_token_index);
            finalize_transaction_cancel_shell(package, instance, now_ms)
        }
        TransactionCompensationCompletionMode::ScopeCompletion => {
            let _ = state::remove_active_token(instance, current_token_index);
            finalize_transaction_scope_completion(package, instance, now_ms)
        }
        TransactionCompensationCompletionMode::IntermediateRouting { node_index } => {
            route_after_intermediate_throw_compensation(
                process,
                instance,
                current_token_index,
                node_index,
                now_ms,
            )
        }
        TransactionCompensationCompletionMode::Detached => {
            let _ = state::remove_active_token(instance, current_token_index);
            if !instance.active_tokens.is_empty() {
                state::record_transition(instance, now_ms, InstanceLifecycle::Running);
                return Ok(());
            }
            finalize_transaction_scope_completion(package, instance, now_ms)
        }
    }
}

pub(super) fn queue_transaction_compensation_targets(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    completed_activity_node_indices: impl IntoIterator<Item = BpmnNodeIndex>,
    now_ms: u64,
    completion_mode: TransactionCompensationCompletionMode,
) -> Result<bool> {
    let Some(frame) = instance.call_stack.last_mut() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "queue_transaction_compensation_missing_parent_frame",
        });
    };
    let Some(state) = frame.transaction_compensation.as_mut() else {
        return Ok(false);
    };

    let mut pending_handler_node_indices = Vec::new();
    for activity_index in completed_activity_node_indices {
        let Some(handler) = process.compensation_handler_for_activity(activity_index) else {
            continue;
        };
        pending_handler_node_indices.push(handler.handler);
    }
    if pending_handler_node_indices.is_empty() {
        return Ok(false);
    }
    let first_handler = pending_handler_node_indices.remove(0);
    pending_handler_node_indices.reverse();
    state.pending_handler_node_indices = pending_handler_node_indices;
    state.cancelling = true;
    state.completion_mode = completion_mode;
    state::set_node_status(instance, first_handler, NodeRuntimeStatus::Queued);
    let _ = state::push_active_token_with_arrival(instance, None, first_handler);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(true)
}

pub(crate) fn record_completed_compensable_activity(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    if process.nodes[node_index as usize].is_for_compensation {
        return;
    }
    if process
        .compensation_handler_for_activity(node_index)
        .is_none()
    {
        return;
    }
    let Some(frame) = instance.call_stack.last_mut() else {
        return;
    };
    let Some(state) = frame.transaction_compensation.as_mut() else {
        return;
    };
    if state.completed_activity_node_indices.contains(&node_index) {
        return;
    }
    state.completed_activity_node_indices.push(node_index);
}

pub(super) fn route_after_intermediate_throw_compensation(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let edge_index = state::resolve_single_outgoing_edge(
        process,
        node_index,
        "complete_compensation_handler_intermediate_throw_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    state::set_active_node_index(instance, current_token_index, edge_index, next_node_index);
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}
