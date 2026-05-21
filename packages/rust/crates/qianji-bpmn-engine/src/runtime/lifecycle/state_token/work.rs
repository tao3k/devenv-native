use crate::runtime::lifecycle::scope::{
    BpmnInstanceState, BpmnNodeIndex, PendingHostWork, PendingHostWorkKind,
};
use crate::runtime_instance_api::{BpmnHumanTaskLifecycleEvent, BpmnHumanTaskLifecycleEventKind};

pub(crate) fn clear_pending_host_work(instance: &mut BpmnInstanceState, token_id: u64) {
    instance
        .pending_host_work
        .retain(|pending| pending.token_id != token_id);
}

pub(crate) fn record_human_task_lifecycle_event(
    instance: &mut BpmnInstanceState,
    kind: BpmnHumanTaskLifecycleEventKind,
    pending: &PendingHostWork,
    occurred_at_ms: u64,
    claimant: Option<String>,
) {
    if !matches!(
        pending.kind,
        PendingHostWorkKind::User | PendingHostWorkKind::Manual
    ) {
        return;
    }

    instance
        .human_task_events
        .push(BpmnHumanTaskLifecycleEvent {
            sequence: next_human_task_lifecycle_event_sequence(instance),
            kind,
            occurred_at_ms,
            process_id: pending
                .process_id
                .clone()
                .unwrap_or_else(|| instance.process.process_id.as_ref().into()),
            activity_id: pending
                .activity_id
                .clone()
                .unwrap_or_else(|| format!("node#{}", pending.node_index).into()),
            token_id: (pending.token_id),
            node_index: pending.node_index,
            work_kind: pending.kind.clone(),
            claimant,
            work_id: pending.work_id.clone(),
        });
}

pub(crate) fn has_pending_host_work_for_process_node(
    instance: &BpmnInstanceState,
    process_id: &str,
    node_index: BpmnNodeIndex,
) -> bool {
    instance.pending_host_work.iter().any(|pending| {
        pending.process_id.as_deref().unwrap_or(process_id) == process_id
            && pending.node_index == node_index
    })
}

pub(crate) fn clear_boundary_wait_for_node(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    instance
        .waits
        .retain(|wait| wait.blocking_node_index != Some(node_index));
}

fn next_human_task_lifecycle_event_sequence(instance: &BpmnInstanceState) -> u64 {
    u64::try_from(instance.human_task_events.len())
        .unwrap_or(u64::MAX)
        .saturating_add(1)
}
