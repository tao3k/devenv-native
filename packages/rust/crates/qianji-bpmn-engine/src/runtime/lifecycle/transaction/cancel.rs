//! Canonical transaction lifecycle owner.
//!
//! The root transaction module re-exports only this facade; detailed
//! responsibilities stay in private sibling modules.

use crate::runtime::lifecycle::scope::{
    BpmnInstanceState, BpmnNodeIndex, BpmnPackage, BpmnProcessSpec, Result,
};

pub(crate) fn cancel_transaction_boundary_siblings(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    owner_node_index: BpmnNodeIndex,
    selected_boundary_indices: &[BpmnNodeIndex],
) -> Result<()> {
    super::boundary::cancel_transaction_boundary_siblings(
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
    super::shell::cancel_transaction_shell(
        package,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )
}

pub(crate) fn error_transaction_shell(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    thrown_reference_id: Option<&str>,
    now_ms: u64,
) -> Result<()> {
    super::error::error_transaction_shell(
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
    target_activity_bpmn_id: Option<&str>,
    now_ms: u64,
) -> Result<()> {
    super::throw::throw_compensation_end_event(
        package,
        process,
        instance,
        current_token_index,
        current_node_index,
        target_activity_bpmn_id,
        now_ms,
    )
}

pub(crate) fn throw_compensation_intermediate_event(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    target_activity_bpmn_id: Option<&str>,
    now_ms: u64,
) -> Result<()> {
    super::throw::throw_compensation_intermediate_event(
        process,
        instance,
        current_token_index,
        current_node_index,
        target_activity_bpmn_id,
        now_ms,
    )
}

pub(crate) fn transaction_compensation_is_running(instance: &BpmnInstanceState) -> bool {
    super::queue::transaction_compensation_is_running(instance)
}

pub(crate) fn complete_compensation_handler(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    super::queue::complete_compensation_handler(
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
    super::queue::record_completed_compensable_activity(process, instance, node_index);
}
