use super::{BpmnParticipantSnapshot, Value, json};

pub(in crate::lint::bpmn::document_surface) fn participant_evidence(
    participant: &BpmnParticipantSnapshot,
) -> Value {
    json!({
        "participant_id": participant.participant_id,
        "name": participant.name,
        "process_ref": participant.process_ref,
        "interface_refs": participant.interface_refs,
        "end_point_refs": participant.end_point_refs,
        "participant_multiplicity": participant.participant_multiplicity.as_ref().map(|multiplicity| {
            json!({
                "multiplicity_id": multiplicity.multiplicity_id,
                "minimum": multiplicity.minimum,
                "maximum": multiplicity.maximum,
            })
        }),
    })
}
