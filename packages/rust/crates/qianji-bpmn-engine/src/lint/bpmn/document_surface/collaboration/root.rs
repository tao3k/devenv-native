use super::{
    BpmnDocumentSnapshot, BpmnPartnerEntitySnapshot, BpmnPartnerRoleSnapshot,
    SNAPSHOT_EVIDENCE_LIMIT, Value, json,
};

pub(in crate::lint::bpmn::document_surface) fn item_definition_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .root
        .item_definitions
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|item_definition| {
            json!({
                "item_definition_id": item_definition.item_definition_id,
                "structure_ref": item_definition.structure_ref,
                "item_kind": item_definition.item_kind,
                "is_collection": item_definition.is_collection,
            })
        })
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn message_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .root
        .messages
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|message| {
            json!({
                "message_id": message.message_id,
                "name": message.name,
                "item_ref": message.item_ref,
            })
        })
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn interface_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .root
        .interfaces
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|interface| {
            json!({
                "interface_id": interface.interface_id,
                "name": interface.name,
                "implementation_ref": interface.implementation_ref,
                "operation_count": interface.operations.len(),
                "operations": interface.operations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|operation| {
                    json!({
                        "operation_id": operation.operation_id,
                        "name": operation.name,
                        "implementation_ref": operation.implementation_ref,
                        "in_message_ref": operation.in_message_ref,
                        "out_message_ref": operation.out_message_ref,
                        "error_refs": operation.error_refs,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn partner_entity_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .root
        .partner_entities
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(partner_entity_item_evidence)
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn partner_entity_item_evidence(
    partner_entity: &BpmnPartnerEntitySnapshot,
) -> Value {
    json!({
        "partner_entity_id": partner_entity.partner_entity_id,
        "name": partner_entity.name,
        "participant_refs": partner_entity.participant_refs,
    })
}

pub(in crate::lint::bpmn::document_surface) fn partner_role_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .root
        .partner_roles
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(partner_role_item_evidence)
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn partner_role_item_evidence(
    partner_role: &BpmnPartnerRoleSnapshot,
) -> Value {
    json!({
        "partner_role_id": partner_role.partner_role_id,
        "name": partner_role.name,
        "participant_refs": partner_role.participant_refs,
    })
}

pub(in crate::lint::bpmn::document_surface) fn correlation_property_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .root
        .correlation_properties
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|property| {
            json!({
                "correlation_property_id": property.correlation_property_id,
                "name": property.name,
                "type_ref": property.type_ref,
                "retrieval_expression_count": property.retrieval_expressions.len(),
                "retrieval_expressions": property.retrieval_expressions.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|retrieval| {
                    json!({
                        "retrieval_expression_id": retrieval.retrieval_expression_id,
                        "message_ref": retrieval.message_ref,
                        "message_path": retrieval.message_path,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect()
}
