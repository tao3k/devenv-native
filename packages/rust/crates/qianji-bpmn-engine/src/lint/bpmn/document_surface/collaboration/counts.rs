use super::{
    BpmnDocumentSnapshot, CollaborationCounts, choreography_activity_count,
    collaboration_correlation_key_count, conversation_node_count,
};

pub(in crate::lint::bpmn::document_surface) fn collaboration_counts(
    snapshot: &BpmnDocumentSnapshot,
) -> CollaborationCounts {
    snapshot.collaborations.iter().fold(
        CollaborationCounts::default(),
        |mut counts, collaboration| {
            counts.participant += collaboration.participants.len();
            for participant in &collaboration.participants {
                counts.participant_interface_ref += participant.interface_refs.len();
                counts.participant_end_point_ref += participant.end_point_refs.len();
                counts.participant_multiplicity +=
                    usize::from(participant.participant_multiplicity.is_some());
            }
            counts.message_flow += collaboration.message_flows.len();
            counts.conversation_node += collaboration
                .conversation_nodes
                .iter()
                .map(conversation_node_count)
                .sum::<usize>();
            counts.conversation_link += collaboration.conversation_links.len();
            counts.conversation_association += collaboration.conversation_associations.len();
            counts.participant_association += collaboration.participant_associations.len();
            counts.message_flow_association += collaboration.message_flow_associations.len();
            counts.correlation_key += collaboration_correlation_key_count(collaboration);
            counts.choreography_activity += collaboration
                .choreography_activities
                .iter()
                .map(choreography_activity_count)
                .sum::<usize>();
            counts.association += collaboration.associations.len();
            counts.group += collaboration.groups.len();
            counts.text_annotation += collaboration.text_annotations.len();
            counts
        },
    )
}
