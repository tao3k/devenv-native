use super::boundary::cancel_transaction_boundary_siblings as cancel_transaction_boundary_siblings_impl;
use super::error::error_transaction_shell as error_transaction_shell_impl;
use super::finalize::{finalize_transaction_cancel_shell, finalize_transaction_scope_completion};
use super::queue::{
    complete_compensation_handler as complete_compensation_handler_impl,
    queue_transaction_compensation_targets,
    transaction_compensation_is_running as transaction_compensation_is_running_impl,
};
use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnPackage, BpmnProcessSpec, Result,
    resolve_process_for_instance,
};
use crate::runtime::lifecycle::state;
use crate::runtime_instance_api::TransactionCompensationCompletionMode;

pub(crate) fn cancel_transaction_boundary_siblings(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    owner_node_index: BpmnNodeIndex,
    selected_boundary_indices: &[BpmnNodeIndex],
) -> Result<()> {
    cancel_transaction_boundary_siblings_impl(
        process,
        instance,
        owner_node_index,
        selected_boundary_indices,
    )
}

pub(crate) fn cancel_transaction_shell(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(
        instance,
        current_node_index,
        crate::NodeRuntimeStatus::Completed,
    );
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
    let completed_activity_node_indices = instance
        .call_stack
        .last()
        .and_then(|frame| frame.transaction_compensation.as_ref())
        .map(|state| state.completed_activity_node_indices.clone())
        .unwrap_or_default();
    if queue_transaction_compensation_targets(
        process,
        instance,
        completed_activity_node_indices.into_iter().rev(),
        now_ms,
        TransactionCompensationCompletionMode::CancelBoundary,
    )? {
        return Ok(());
    }
    finalize_transaction_cancel_shell(package, instance, now_ms)
}

pub(crate) fn error_transaction_shell(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    thrown_reference_id: Option<&str>,
    now_ms: u64,
) -> Result<()> {
    error_transaction_shell_impl(
        package,
        instance,
        current_token_index,
        current_node_index,
        thrown_reference_id,
        now_ms,
    )
}

pub(crate) fn throw_compensation_end_event(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    target_activity_bpmn_id: &str,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(
        instance,
        current_node_index,
        crate::NodeRuntimeStatus::Completed,
    );
    let _ = state::remove_active_token(instance, current_token_index);

    let target_activity_index = process
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == target_activity_bpmn_id)
        .map(|node| node.index)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "throw_compensation_end_event_missing_target_activity",
        })?;

    if queue_transaction_compensation_targets(
        process,
        instance,
        std::iter::once(target_activity_index),
        now_ms,
        TransactionCompensationCompletionMode::ScopeCompletion,
    )? {
        return Ok(());
    }

    finalize_transaction_scope_completion(package, instance, now_ms)
}

pub(crate) fn throw_compensation_intermediate_event(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    target_activity_bpmn_id: &str,
    now_ms: u64,
) -> Result<()> {
    let target_activity_index = process
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == target_activity_bpmn_id)
        .map(|node| node.index)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "throw_compensation_intermediate_event_missing_target_activity",
        })?;

    state::set_node_status(
        instance,
        current_node_index,
        crate::NodeRuntimeStatus::Completed,
    );
    let _ = state::remove_active_token(instance, current_token_index);

    if queue_transaction_compensation_targets(
        process,
        instance,
        std::iter::once(target_activity_index),
        now_ms,
        TransactionCompensationCompletionMode::IntermediateRouting {
            node_index: current_node_index,
        },
    )? {
        return Ok(());
    }

    Err(BpmnEngineError::UnsupportedOperation {
        operation: "throw_compensation_intermediate_event_missing_handler_queue",
    })
}

pub(crate) fn transaction_compensation_is_running(instance: &BpmnInstanceState) -> bool {
    transaction_compensation_is_running_impl(instance)
}

pub(crate) fn complete_compensation_handler(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    complete_compensation_handler_impl(
        package,
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )
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
