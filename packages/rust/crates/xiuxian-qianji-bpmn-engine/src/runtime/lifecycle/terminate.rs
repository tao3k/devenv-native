use super::scope::{
    BpmnAdvanceOutcome, BpmnInstanceState, BpmnNodeIndex, BpmnPackage, InstanceLifecycle,
    NodeRuntimeStatus, Result,
};
use super::{call_activity, state};

pub(crate) fn terminate_end_event(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    cancel_live_scope_nodes(instance, current_node_index);
    state::set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    let _ = state::remove_active_token(instance, current_token_index);
    clear_live_scope_runtime(instance);

    if instance.call_stack.is_empty() {
        state::record_transition(instance, now_ms, InstanceLifecycle::Completed);
        return Ok(Some(BpmnAdvanceOutcome::Completed));
    }

    call_activity::complete_call_activity(package, instance, now_ms)?;
    Ok(None)
}

fn cancel_live_scope_nodes(instance: &mut BpmnInstanceState, current_node_index: BpmnNodeIndex) {
    let mut cancelled_node_indices = instance
        .active_tokens
        .iter()
        .map(|token| token.node_index)
        .filter(|node_index| *node_index != current_node_index)
        .collect::<Vec<_>>();
    cancelled_node_indices.extend(
        instance
            .waits
            .iter()
            .flat_map(|wait| [Some(wait.node_index), wait.blocking_node_index])
            .flatten()
            .filter(|node_index| *node_index != current_node_index),
    );
    cancelled_node_indices.extend(
        instance
            .pending_host_work
            .iter()
            .map(|pending| pending.node_index)
            .filter(|node_index| *node_index != current_node_index),
    );
    cancelled_node_indices.sort_unstable();
    cancelled_node_indices.dedup();
    for node_index in cancelled_node_indices {
        state::set_node_status(instance, node_index, NodeRuntimeStatus::Cancelled);
    }
}

fn clear_live_scope_runtime(instance: &mut BpmnInstanceState) {
    instance.active_tokens.clear();
    instance.joins.clear();
    instance.standard_loops.clear();
    instance.sequential_multi_instances.clear();
    instance.parallel_multi_instances.clear();
    instance.waits.clear();
    instance.event_competition = None;
    instance.detached_transaction_compensation = None;
    instance.pending_host_work.clear();
    instance.suspend_reason = None;
}
