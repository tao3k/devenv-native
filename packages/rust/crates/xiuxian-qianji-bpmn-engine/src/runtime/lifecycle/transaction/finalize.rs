use crate::runtime::lifecycle::boundary::cancel_attached_boundary_siblings;
use crate::runtime::lifecycle::call_activity;
use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnEventKind, BpmnInstanceState, BpmnPackage, BpmnSubProcessKind,
    InstanceLifecycle, NodeRuntimeStatus, Result, pop_call_activity_frame,
    resolve_process_for_instance, restore_call_activity_frame,
};
use crate::runtime::lifecycle::state;

pub(super) fn finalize_transaction_cancel_shell(
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
    cancel_attached_boundary_siblings(process, instance, return_node_index, &[boundary.index])?;
    state::clear_boundary_wait_for_node(instance, return_node_index);
    if instance.waits.is_empty() {
        instance.suspend_reason = None;
    }
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

pub(super) fn finalize_transaction_scope_completion(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    now_ms: u64,
) -> Result<()> {
    if instance.call_stack.is_empty() {
        instance.pending_host_work.clear();
        instance.waits.clear();
        instance.suspend_reason = None;
        state::record_transition(instance, now_ms, InstanceLifecycle::Completed);
        return Ok(());
    }

    call_activity::complete_call_activity(package, instance, now_ms)
}
