use super::token_cursor::{TokenIdAllocator, allocate_token_id};
use super::trace::push_flow_take_trace;
use crate::runtime::lifecycle::scope::{
    BpmnInstanceState, BpmnNodeIndex, InclusiveJoinHint, TokenRecord,
};

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

fn push_active_token_with_metadata(
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
