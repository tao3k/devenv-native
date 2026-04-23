//! Canonical transaction lifecycle owner.
//!
//! The root transaction module re-exports only this facade; detailed
//! responsibilities stay in private sibling modules.

use crate::runtime::lifecycle::scope::{
    BpmnInstanceState, BpmnNodeIndex, BpmnPackage, BpmnProcessSpec, Result,
};

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

pub(crate) fn throw_compensation_end_event_async(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    target_activity_bpmn_id: Option<&str>,
    now_ms: u64,
) -> Result<()> {
    super::throw::throw_compensation_end_event_async(
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

pub(crate) fn throw_compensation_intermediate_event_async(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    target_activity_bpmn_id: Option<&str>,
    now_ms: u64,
) -> Result<()> {
    super::throw::throw_compensation_intermediate_event_async(
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

pub(crate) fn detached_compensation_matches_pending(
    instance: &BpmnInstanceState,
    pending: &crate::runtime::PendingHostWork,
) -> bool {
    super::detached::detached_compensation_matches_pending(instance, pending)
}

pub(crate) fn complete_detached_compensation_handler(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    now_ms: u64,
) -> Result<()> {
    super::detached::continue_detached_compensation_queue(package, instance, now_ms)
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
