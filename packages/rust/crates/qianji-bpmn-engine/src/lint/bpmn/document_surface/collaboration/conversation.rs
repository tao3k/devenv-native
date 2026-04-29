use super::{
    BpmnCollaborationSnapshot, BpmnConversationNodeSnapshot, Value,
    choreography_activity_correlation_key_count, json,
};

pub(in crate::lint::bpmn::document_surface) fn conversation_node_count(
    node: &BpmnConversationNodeSnapshot,
) -> usize {
    1 + node
        .child_nodes
        .iter()
        .map(conversation_node_count)
        .sum::<usize>()
}

pub(in crate::lint::bpmn::document_surface) fn collaboration_correlation_key_count(
    collaboration: &BpmnCollaborationSnapshot,
) -> usize {
    collaboration.correlation_keys.len()
        + collaboration
            .conversation_nodes
            .iter()
            .map(conversation_node_correlation_key_count)
            .sum::<usize>()
        + collaboration
            .choreography_activities
            .iter()
            .map(choreography_activity_correlation_key_count)
            .sum::<usize>()
}

pub(in crate::lint::bpmn::document_surface) fn conversation_node_correlation_key_count(
    node: &BpmnConversationNodeSnapshot,
) -> usize {
    node.correlation_keys.len()
        + node
            .child_nodes
            .iter()
            .map(conversation_node_correlation_key_count)
            .sum::<usize>()
}

pub(in crate::lint::bpmn::document_surface) fn conversation_node_evidence(
    node: &BpmnConversationNodeSnapshot,
) -> Value {
    json!({
        "node_kind": node.node_kind,
        "node_id": node.node_id,
        "called_collaboration_ref": node.called_collaboration_ref,
        "participant_refs": node.participant_refs,
        "message_flow_refs": node.message_flow_refs,
        "correlation_key_count": conversation_node_correlation_key_count(node),
        "participant_association_count": node.participant_associations.len(),
        "child_node_count": node.child_nodes.iter().map(conversation_node_count).sum::<usize>(),
    })
}
