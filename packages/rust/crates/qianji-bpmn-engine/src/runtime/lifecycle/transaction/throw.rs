use super::detached::{continue_detached_compensation_queue, install_detached_compensation_queue};
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

    let queued = queue_throw_compensation_targets(
        process,
        instance,
        target_activity_bpmn_id,
        now_ms,
        TransactionCompensationCompletionMode::ScopeCompletion,
        "throw_compensation_end_event_missing_target_activity",
    )?;

    if queued {
        return Ok(());
    }

    finalize_transaction_scope_completion(package, instance, now_ms)
}

pub(crate) fn throw_compensation_end_event_async(
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

    let pending_handler_node_indices = collect_throw_compensation_handler_node_indices(
        process,
        instance,
        target_activity_bpmn_id,
        "throw_compensation_end_event_missing_target_activity",
    )?;
    if target_activity_bpmn_id.is_some() && pending_handler_node_indices.is_empty() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "throw_compensation_end_event_missing_handler_queue",
        });
    }
    if pending_handler_node_indices.is_empty() {
        return finalize_transaction_scope_completion(package, instance, now_ms);
    }

    install_detached_compensation_queue(instance, process, pending_handler_node_indices)?;
    finalize_transaction_scope_completion(package, instance, now_ms)?;
    continue_detached_compensation_queue(package, instance, now_ms)
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

    let queued = queue_throw_compensation_targets(
        process,
        instance,
        target_activity_bpmn_id,
        now_ms,
        TransactionCompensationCompletionMode::IntermediateRouting {
            node_index: current_node_index,
        },
        "throw_compensation_intermediate_event_missing_target_activity",
    )?;

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

pub(crate) fn throw_compensation_intermediate_event_async(
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

    let queued = queue_throw_compensation_targets(
        process,
        instance,
        target_activity_bpmn_id,
        now_ms,
        TransactionCompensationCompletionMode::Detached,
        "throw_compensation_intermediate_event_missing_target_activity",
    )?;

    if target_activity_bpmn_id.is_some() && !queued {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "throw_compensation_intermediate_event_missing_handler_queue",
        });
    }

    route_after_intermediate_throw_compensation(
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )
}

fn completed_activity_node_indices(instance: &BpmnInstanceState) -> Vec<BpmnNodeIndex> {
    instance
        .call_stack
        .last()
        .and_then(|frame| frame.transaction_compensation.as_ref())
        .map(|state| state.completed_activity_node_indices.clone())
        .unwrap_or_default()
}

fn queue_throw_compensation_targets(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    target_activity_bpmn_id: Option<&str>,
    now_ms: u64,
    completion_mode: TransactionCompensationCompletionMode,
    missing_target_operation: &'static str,
) -> Result<bool> {
    let completed_activity_node_indices = collect_throw_compensation_activity_node_indices(
        process,
        instance,
        target_activity_bpmn_id,
        missing_target_operation,
    )?;
    queue_transaction_compensation_targets(
        process,
        instance,
        completed_activity_node_indices,
        now_ms,
        completion_mode,
    )
}

fn collect_throw_compensation_activity_node_indices(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    target_activity_bpmn_id: Option<&str>,
    missing_target_operation: &'static str,
) -> Result<Vec<BpmnNodeIndex>> {
    if let Some(target_activity_bpmn_id) = target_activity_bpmn_id {
        let target_activity_index = process
            .nodes
            .iter()
            .find(|node| node.bpmn_id.as_ref() == target_activity_bpmn_id)
            .map(|node| node.index)
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: missing_target_operation,
            })?;
        return Ok(vec![target_activity_index]);
    }

    Ok(completed_activity_node_indices(instance)
        .into_iter()
        .rev()
        .collect())
}

fn collect_throw_compensation_handler_node_indices(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    target_activity_bpmn_id: Option<&str>,
    missing_target_operation: &'static str,
) -> Result<Vec<BpmnNodeIndex>> {
    if let Some(target_activity_bpmn_id) = target_activity_bpmn_id {
        let target_activity_index = process
            .nodes
            .iter()
            .find(|node| node.bpmn_id.as_ref() == target_activity_bpmn_id)
            .map(|node| node.index)
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: missing_target_operation,
            })?;
        return Ok(process
            .compensation_handler_for_activity(target_activity_index)
            .map(|handler| vec![handler.handler])
            .unwrap_or_default());
    }

    Ok(completed_activity_node_indices(instance)
        .into_iter()
        .rev()
        .filter_map(|activity_index| {
            process
                .compensation_handler_for_activity(activity_index)
                .map(|handler| handler.handler)
        })
        .collect())
}
