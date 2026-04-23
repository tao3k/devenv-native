use super::finalize::finalize_transaction_cancel_shell;
use super::queue::queue_transaction_compensation_targets;
use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnPackage, Result,
    resolve_process_for_instance,
};
use crate::runtime::lifecycle::state;
use crate::runtime_instance_api::TransactionCompensationCompletionMode;

pub(super) fn cancel_transaction_shell(
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
