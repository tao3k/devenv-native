pub(crate) use super::boundary::cancel_transaction_boundary_siblings;
pub(crate) use super::error::error_transaction_shell;
use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnEventKind, BpmnInstanceState, BpmnNodeIndex, BpmnPackage, BpmnProcessSpec,
    BpmnSubProcessKind, InstanceLifecycle, NodeRuntimeStatus, Result, pop_call_activity_frame,
    resolve_process_for_instance, restore_call_activity_frame,
};
use crate::runtime::lifecycle::state;

pub(crate) fn cancel_transaction_shell(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    let _ = state::remove_active_token(instance, current_token_index);

    let transaction_cancel_variables = instance
        .call_stack
        .last()
        .and_then(|frame| frame.transaction_cancel_variables.clone());
    let Some(transaction_cancel_variables) = transaction_cancel_variables else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_missing_variable_snapshot",
        });
    };
    instance.variables = transaction_cancel_variables;

    let process = resolve_process_for_instance(package, instance)?;
    if queue_transaction_compensation(process, instance, now_ms)? {
        return Ok(());
    }
    finalize_transaction_cancel_shell(package, instance, now_ms)
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
    let _ = state::remove_active_token(instance, current_token_index);
    let next_handler = instance
        .call_stack
        .last_mut()
        .and_then(|frame| frame.transaction_compensation.as_mut())
        .and_then(|state| state.pending_handler_node_indices.pop());

    if let Some(next_handler) = next_handler {
        state::set_node_status(instance, next_handler, NodeRuntimeStatus::Queued);
        let _ = state::push_active_token_with_arrival(instance, None, next_handler);
        state::record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(());
    }

    finalize_transaction_cancel_shell(package, instance, now_ms)
}

fn queue_transaction_compensation(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    now_ms: u64,
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
    for activity_index in state.completed_activity_node_indices.iter().rev() {
        let Some(handler) = process.compensation_handler_for_activity(*activity_index) else {
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
    state::set_node_status(instance, first_handler, NodeRuntimeStatus::Queued);
    let _ = state::push_active_token_with_arrival(instance, None, first_handler);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(true)
}

fn finalize_transaction_cancel_shell(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    now_ms: u64,
) -> Result<()> {
    let frame = pop_call_activity_frame(instance).ok_or(BpmnEngineError::UnsupportedOperation {
        operation: "cancel_transaction_shell_missing_parent_frame",
    })?;
    let transaction_cancel_variables = frame.transaction_cancel_variables.clone();
    let return_node_index = restore_call_activity_frame(instance, frame);
    let Some(transaction_cancel_variables) = transaction_cancel_variables else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_missing_variable_snapshot",
        });
    };
    instance.variables = transaction_cancel_variables;

    let process = resolve_process_for_instance(package, instance)?;
    let transaction_node = process.nodes.get(return_node_index as usize).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_missing_parent_node",
        },
    )?;
    if transaction_node.subprocess_kind != Some(BpmnSubProcessKind::Transaction) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_parent_not_transaction",
        });
    }

    let Some(boundary) = process
        .boundary_events_for_attached_node(return_node_index)
        .find(|boundary| {
            process
                .event_for_node(boundary.index)
                .is_some_and(|event| event.kind == BpmnEventKind::Cancel)
        })
    else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_missing_boundary",
        });
    };

    let parent_token_index = state::token_index_for_node(instance, return_node_index).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "cancel_transaction_shell_missing_parent_token",
        },
    )?;
    state::set_node_status(instance, return_node_index, NodeRuntimeStatus::Cancelled);
    cancel_transaction_boundary_siblings(process, instance, return_node_index, &[boundary.index])?;
    state::set_node_status(instance, boundary.index, NodeRuntimeStatus::Completed);
    let edge_index = state::resolve_single_outgoing_edge(
        process,
        boundary.index,
        "cancel_transaction_shell_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    state::set_active_node_index(instance, parent_token_index, edge_index, next_node_index);
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}
