use super::{BpmnChoreographyActivitySnapshot, Value, json};

pub(in crate::lint::bpmn::document_surface) fn choreography_activity_count(
    activity: &BpmnChoreographyActivitySnapshot,
) -> usize {
    1 + activity
        .child_activities
        .iter()
        .map(choreography_activity_count)
        .sum::<usize>()
}

pub(in crate::lint::bpmn::document_surface) fn choreography_activity_correlation_key_count(
    activity: &BpmnChoreographyActivitySnapshot,
) -> usize {
    activity.correlation_keys.len()
        + activity
            .child_activities
            .iter()
            .map(choreography_activity_correlation_key_count)
            .sum::<usize>()
}

pub(in crate::lint::bpmn::document_surface) fn choreography_activity_evidence(
    activity: &BpmnChoreographyActivitySnapshot,
) -> Value {
    json!({
        "activity_kind": activity.activity_kind,
        "activity_id": activity.activity_id,
        "initiating_participant_ref": activity.initiating_participant_ref,
        "loop_type": activity.loop_type,
        "called_choreography_ref": activity.called_choreography_ref,
        "participant_refs": activity.participant_refs,
        "message_flow_refs": activity.message_flow_refs,
        "correlation_key_count": choreography_activity_correlation_key_count(activity),
        "participant_association_count": activity.participant_associations.len(),
        "child_activity_count": activity.child_activities.iter().map(choreography_activity_count).sum::<usize>(),
    })
}
