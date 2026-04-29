use super::{BpmnAssociationSnapshot, BpmnGroupSnapshot, BpmnTextAnnotationSnapshot, Value, json};

pub(in crate::lint::bpmn::document_surface) fn artifact_association_evidence(
    association: &BpmnAssociationSnapshot,
) -> Value {
    json!({
        "association_id": association.association_id,
        "source_ref": association.source_ref,
        "target_ref": association.target_ref,
        "association_direction": association.association_direction,
    })
}

pub(in crate::lint::bpmn::document_surface) fn artifact_group_evidence(
    group: &BpmnGroupSnapshot,
) -> Value {
    json!({
        "group_id": group.group_id,
        "category_value_ref": group.category_value_ref,
    })
}

pub(in crate::lint::bpmn::document_surface) fn text_annotation_evidence(
    annotation: &BpmnTextAnnotationSnapshot,
) -> Value {
    json!({
        "annotation_id": annotation.annotation_id,
        "text_format": annotation.text_format,
        "text": annotation.text,
    })
}
