use super::token_cursor::{TokenIdAllocator, allocate_token_id};
use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnExecutionTraceEvent, BpmnExecutionTraceEventKind, BpmnInstanceState,
    BpmnNodeIndex, BpmnNodeKind, BpmnProcessSpec, InclusiveJoinHint, InstanceLifecycle,
    NodeRuntimeStatus, PendingHostWork, PendingHostWorkKind, Result, TokenRecord,
};
use crate::runtime_instance_api::{BpmnHumanTaskLifecycleEvent, BpmnHumanTaskLifecycleEventKind};

pub(crate) fn find_single_start_node(process: &BpmnProcessSpec) -> Result<BpmnNodeIndex> {
    let mut start_nodes = process
        .nodes
        .iter()
        .filter(|node| node.kind == BpmnNodeKind::StartEvent)
        .map(|node| node.index);
    let Some(start_node_index) = start_nodes.next() else {
        return Err(BpmnEngineError::MissingRequiredProcessElement {
            process_id: process.key.process_id.to_string(),
            element: "start event",
        });
    };
    if start_nodes.next().is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_multiple_start_events",
        });
    }
    Ok(start_node_index)
}

pub(crate) fn set_active_node_index(
    instance: &mut BpmnInstanceState,
    token_index: usize,
    incoming_edge_index: u32,
    node_index: BpmnNodeIndex,
) {
    let token_exists = instance.active_tokens.get(token_index).is_some();
    if token_exists {
        push_flow_take_trace(instance, incoming_edge_index, node_index);
    }
    if let Some(token) = instance.active_tokens.get_mut(token_index) {
        token.node_index = node_index;
        token.incoming_edge_index = Some(incoming_edge_index);
    }
}

pub(crate) fn remove_active_token(
    instance: &mut BpmnInstanceState,
    token_index: usize,
) -> Option<TokenRecord> {
    if token_index >= instance.active_tokens.len() {
        None
    } else {
        Some(instance.active_tokens.remove(token_index))
    }
}

pub(crate) fn push_active_token(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: u32,
    node_index: BpmnNodeIndex,
) {
    let _ = push_active_token_with_arrival(instance, Some(incoming_edge_index), node_index);
}

pub(crate) fn push_active_token_with_allocator(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: u32,
    node_index: BpmnNodeIndex,
    allocator: &mut TokenIdAllocator,
) -> u64 {
    let token_id = allocator.next_token_id();
    allocator.reserve_on(instance);
    push_active_token_with_metadata_and_id(
        instance,
        Some(incoming_edge_index),
        node_index,
        None,
        token_id,
    )
}

pub(crate) fn push_active_token_with_arrival_and_allocator(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: Option<u32>,
    node_index: BpmnNodeIndex,
    allocator: &mut TokenIdAllocator,
) -> u64 {
    let token_id = allocator.next_token_id();
    allocator.reserve_on(instance);
    push_active_token_with_metadata_and_id(
        instance,
        incoming_edge_index,
        node_index,
        None,
        token_id,
    )
}

pub(crate) fn push_active_token_with_join_hint_and_allocator(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: u32,
    node_index: BpmnNodeIndex,
    inclusive_join_hint: InclusiveJoinHint,
    allocator: &mut TokenIdAllocator,
) -> u64 {
    let token_id = allocator.next_token_id();
    allocator.reserve_on(instance);
    push_active_token_with_metadata_and_id(
        instance,
        Some(incoming_edge_index),
        node_index,
        Some(inclusive_join_hint),
        token_id,
    )
}

pub(crate) fn push_active_token_with_arrival(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: Option<u32>,
    node_index: BpmnNodeIndex,
) -> u64 {
    push_active_token_with_metadata(instance, incoming_edge_index, node_index, None)
}

pub(crate) fn push_active_token_with_metadata(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: Option<u32>,
    node_index: BpmnNodeIndex,
    inclusive_join_hint: Option<InclusiveJoinHint>,
) -> u64 {
    let token_id = allocate_token_id(instance);
    push_active_token_with_metadata_and_id(
        instance,
        incoming_edge_index,
        node_index,
        inclusive_join_hint,
        token_id,
    )
}

fn push_active_token_with_metadata_and_id(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: Option<u32>,
    node_index: BpmnNodeIndex,
    inclusive_join_hint: Option<InclusiveJoinHint>,
    token_id: u64,
) -> u64 {
    if let Some(edge_index) = incoming_edge_index {
        push_flow_take_trace(instance, edge_index, node_index);
    }
    instance.active_tokens.push(TokenRecord {
        token_id,
        node_index,
        incoming_edge_index,
        inclusive_join_hint,
    });
    token_id
}

pub(crate) fn set_token_inclusive_join_hint(
    instance: &mut BpmnInstanceState,
    token_index: usize,
    inclusive_join_hint: Option<InclusiveJoinHint>,
) {
    if let Some(token) = instance.active_tokens.get_mut(token_index) {
        token.inclusive_join_hint = inclusive_join_hint;
    }
}

pub(crate) fn resolve_single_outgoing_edge(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    operation: &'static str,
) -> Result<u32> {
    let outgoing = process.outgoing_edge_indices(node_index);
    if outgoing.len() != 1 {
        return Err(BpmnEngineError::UnsupportedOperation { operation });
    }
    Ok(outgoing[0])
}

pub(crate) fn set_node_status(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    status: NodeRuntimeStatus,
) {
    let changed = if let Some(node_state) = instance.node_states.get_mut(node_index as usize) {
        if node_state.status == status {
            false
        } else {
            node_state.status = status.clone();
            true
        }
    } else {
        false
    };
    if changed {
        push_node_status_trace(instance, node_index, status);
    }
}

pub(crate) fn record_transition(
    instance: &mut BpmnInstanceState,
    now_ms: u64,
    lifecycle: InstanceLifecycle,
) {
    instance.sequence += 1;
    instance.lifecycle = lifecycle;
    instance.updated_at_ms = now_ms;
}

pub(crate) fn token_index_for_id(instance: &BpmnInstanceState, token_id: u64) -> Option<usize> {
    instance
        .active_tokens
        .iter()
        .position(|token| token.token_id == token_id)
}

pub(crate) fn token_index_for_node(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Option<usize> {
    instance
        .active_tokens
        .iter()
        .position(|token| token.node_index == node_index)
}

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
                .unwrap_or_else(|| instance.process.process_id.to_string()),
            activity_id: pending
                .activity_id
                .clone()
                .unwrap_or_else(|| format!("node#{}", pending.node_index)),
            token_id: pending.token_id,
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

fn push_node_status_trace(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    status: NodeRuntimeStatus,
) {
    instance.trace.push(BpmnExecutionTraceEvent {
        sequence: next_trace_sequence(instance),
        process: instance.process.clone(),
        kind: BpmnExecutionTraceEventKind::NodeStatus,
        node_index: Some(node_index),
        edge_index: None,
        status: Some(status),
    });
}

fn push_flow_take_trace(
    instance: &mut BpmnInstanceState,
    edge_index: u32,
    target_node_index: BpmnNodeIndex,
) {
    instance.trace.push(BpmnExecutionTraceEvent {
        sequence: next_trace_sequence(instance),
        process: instance.process.clone(),
        kind: BpmnExecutionTraceEventKind::FlowTake,
        node_index: Some(target_node_index),
        edge_index: Some(edge_index),
        status: None,
    });
}

fn next_trace_sequence(instance: &BpmnInstanceState) -> u64 {
    u64::try_from(instance.trace.len())
        .unwrap_or(u64::MAX)
        .saturating_add(1)
}

fn next_human_task_lifecycle_event_sequence(instance: &BpmnInstanceState) -> u64 {
    u64::try_from(instance.human_task_events.len())
        .unwrap_or(u64::MAX)
        .saturating_add(1)
}
