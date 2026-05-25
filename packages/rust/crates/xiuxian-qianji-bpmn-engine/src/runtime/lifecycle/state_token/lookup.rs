use crate::runtime::lifecycle::scope::{BpmnInstanceState, BpmnNodeIndex};

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
