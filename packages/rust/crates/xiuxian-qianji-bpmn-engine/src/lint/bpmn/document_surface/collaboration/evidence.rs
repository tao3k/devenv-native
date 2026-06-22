use super::{
    BpmnCollaborationSnapshot, SNAPSHOT_EVIDENCE_LIMIT, Value, artifact_association_evidence,
    artifact_group_evidence, choreography_activity_count, choreography_activity_evidence,
    collaboration_correlation_key_count, conversation_node_count, conversation_node_evidence, json,
    participant_evidence, text_annotation_evidence,
};

pub(in crate::lint::bpmn::document_surface) fn collaboration_evidence(
    collaboration: &BpmnCollaborationSnapshot,
) -> Value {
    json!({
        "collaboration_id": collaboration.collaboration_id,
        "participant_count": collaboration.participants.len(),
        "participant_interface_ref_count": collaboration.participants.iter().map(|participant| participant.interface_refs.len()).sum::<usize>(),
        "participant_end_point_ref_count": collaboration.participants.iter().map(|participant| participant.end_point_refs.len()).sum::<usize>(),
        "participant_multiplicity_count": collaboration.participants.iter().filter(|participant| participant.participant_multiplicity.is_some()).count(),
        "message_flow_count": collaboration.message_flows.len(),
        "conversation_node_count": collaboration.conversation_nodes.iter().map(conversation_node_count).sum::<usize>(),
        "conversation_link_count": collaboration.conversation_links.len(),
        "conversation_association_count": collaboration.conversation_associations.len(),
        "participant_association_count": collaboration.participant_associations.len(),
        "message_flow_association_count": collaboration.message_flow_associations.len(),
        "correlation_key_count": collaboration_correlation_key_count(collaboration),
        "choreography_ref_count": collaboration.choreography_refs.len(),
        "choreography_activity_count": collaboration.choreography_activities.iter().map(choreography_activity_count).sum::<usize>(),
        "artifact_association_count": collaboration.associations.len(),
        "artifact_group_count": collaboration.groups.len(),
        "text_annotation_count": collaboration.text_annotations.len(),
        "initiating_participant_ref": collaboration.initiating_participant_ref,
        "participants": collaboration.participants.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(participant_evidence).collect::<Vec<_>>(),
        "message_flows": collaboration.message_flows.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|flow| {
            json!({
                "message_flow_id": flow.message_flow_id,
                "source_ref": flow.source_ref,
                "target_ref": flow.target_ref,
                "message_ref": flow.message_ref,
            })
        }).collect::<Vec<_>>(),
        "conversation_nodes": collaboration.conversation_nodes.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(conversation_node_evidence).collect::<Vec<_>>(),
        "choreography_activities": collaboration.choreography_activities.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(choreography_activity_evidence).collect::<Vec<_>>(),
        "associations": collaboration.associations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(artifact_association_evidence).collect::<Vec<_>>(),
        "groups": collaboration.groups.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(artifact_group_evidence).collect::<Vec<_>>(),
        "text_annotations": collaboration.text_annotations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(text_annotation_evidence).collect::<Vec<_>>(),
        "conversation_links": collaboration.conversation_links.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|link| {
            json!({
                "link_id": link.link_id,
                "source_ref": link.source_ref,
                "target_ref": link.target_ref,
            })
        }).collect::<Vec<_>>(),
    })
}
