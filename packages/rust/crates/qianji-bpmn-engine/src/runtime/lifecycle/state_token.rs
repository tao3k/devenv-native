use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnNodeKind, BpmnProcessSpec,
    InclusiveJoinHint, InstanceLifecycle, NodeRuntimeStatus, Result, TokenRecord,
};

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

pub(crate) fn push_active_token_with_join_hint(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: u32,
    node_index: BpmnNodeIndex,
    inclusive_join_hint: InclusiveJoinHint,
) {
    let _ = push_active_token_with_metadata(
        instance,
        Some(incoming_edge_index),
        node_index,
        Some(inclusive_join_hint),
    );
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
    let token_id = next_token_id(instance);
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
    if let Some(node_state) = instance.node_states.get_mut(node_index as usize) {
        node_state.status = status;
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

pub(crate) fn has_pending_host_work_for_node(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> bool {
    instance
        .pending_host_work
        .iter()
        .any(|pending| pending.node_index == node_index)
}

pub(crate) fn clear_boundary_wait_for_node(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    instance
        .waits
        .retain(|wait| wait.blocking_node_index != Some(node_index));
}

pub(super) fn next_token_id(instance: &BpmnInstanceState) -> u64 {
    instance
        .active_tokens
        .iter()
        .map(|token| token.token_id)
        .max()
        .unwrap_or(instance.sequence)
        .max(instance.sequence)
        + 1
}
