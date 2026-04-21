use super::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnPackage, BpmnProcessSpec,
    BpmnSubProcessKind, InstanceLifecycle, NodeRuntimeStatus, Result, SuspendReason, TokenRecord,
    install_process_state, pop_call_activity_frame, push_call_activity_frame,
    resolve_process_for_instance, restore_call_activity_frame,
};
use super::{blocking, completion, state, transaction};

pub(super) fn can_bootstrap_start_token(instance: &BpmnInstanceState) -> bool {
    instance.sequence == 0
        && matches!(instance.lifecycle, InstanceLifecycle::Ready)
        && instance
            .node_states
            .iter()
            .all(|state| state.status == NodeRuntimeStatus::Idle)
}

pub(super) fn register_intermediate_wait(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
    instance
        .waits
        .retain(|wait| wait.node_index != node_index || wait.blocking_node_index.is_some());
    instance.waits.push(blocking::build_wait_registration(
        process, node_index, None,
    )?);
    instance.suspend_reason = Some(SuspendReason::ExternalWait);
    state::record_transition(instance, now_ms, InstanceLifecycle::Waiting);

    Ok(())
}

pub(super) fn bootstrap_start_token(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    now_ms: u64,
) -> Result<()> {
    let start_node_index = state::find_single_start_node(process)?;
    instance.active_tokens.push(TokenRecord {
        token_id: instance.sequence + 1,
        node_index: start_node_index,
        incoming_edge_index: None,
    });
    state::set_node_status(instance, start_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

pub(super) fn enter_call_activity(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let node = &process.nodes[node_index as usize];
    let called_process_id = node.called_process_id.as_ref().ok_or_else(|| {
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            detail: "missing_called_process",
        }
    })?;

    if called_process_id.as_ref() == process.key.process_id.as_ref()
        || instance
            .call_stack
            .iter()
            .any(|frame| frame.process.process_id.as_ref() == called_process_id.as_ref())
    {
        return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            detail: "recursive_call_activity",
        });
    }

    let (called_process_index, called_process) = package
        .find_process_position(called_process_id.as_ref())
        .ok_or_else(|| BpmnEngineError::UnknownCalledProcess {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            called_process_id: called_process_id.to_string(),
        })?;

    state::set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
    let transaction_cancel_variables = (node.subprocess_kind
        == Some(BpmnSubProcessKind::Transaction))
    .then(|| instance.variables.clone());
    push_call_activity_frame(instance, node_index, transaction_cancel_variables);
    install_process_state(instance, called_process, called_process_index);
    let _ = state::remove_active_token(instance, current_token_index);
    bootstrap_start_token(called_process, instance, now_ms)?;
    Ok(())
}

pub(super) fn complete_call_activity(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    now_ms: u64,
) -> Result<()> {
    let frame = pop_call_activity_frame(instance).ok_or(BpmnEngineError::UnsupportedOperation {
        operation: "complete_call_activity_missing_parent_frame",
    })?;
    let return_node_index = restore_call_activity_frame(instance, frame);
    let process = resolve_process_for_instance(package, instance)?;
    if process.nodes[return_node_index as usize].subprocess_kind
        == Some(BpmnSubProcessKind::Transaction)
    {
        transaction::cancel_transaction_boundary_siblings(
            process,
            instance,
            return_node_index,
            &[],
        )?;
    }

    completion::complete_node_and_route(
        process,
        instance,
        state::token_index_for_node(instance, return_node_index).ok_or(
            BpmnEngineError::UnsupportedOperation {
                operation: "complete_call_activity_missing_parent_token",
            },
        )?,
        return_node_index,
        now_ms,
        "complete_call_activity_routing",
    )?;
    Ok(())
}
