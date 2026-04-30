use super::{
    BpmnDocumentSnapshot, BpmnSourceFile, SNAPSHOT_EVIDENCE_LIMIT, Value, collaboration_counts,
    collaboration_evidence, correlation_property_evidence, data_snapshot_summary,
    flow_element_metadata_summary, interface_evidence, item_definition_evidence, json,
    message_evidence, partner_entity_evidence, partner_role_evidence, process_callable_summary,
    resource_role_summary, routing_boundary_evidence, snapshot_bpmn_source,
};

pub(in crate::lint::bpmn::document_surface) fn document_surface_evidence(
    source: &BpmnSourceFile,
    tag: &str,
    family: &str,
) -> Value {
    let Ok(snapshot) = snapshot_bpmn_source(source) else {
        return json!({
            "source_id": source.source_id,
            "element": tag,
            "deferred_family": family,
            "snapshot_available": false,
        });
    };

    json!({
        "source_id": source.source_id,
        "element": tag,
        "deferred_family": family,
        "snapshot_available": true,
        "snapshot": snapshot_family_summary(&snapshot, family),
    })
}

pub(super) fn snapshot_family_summary(snapshot: &BpmnDocumentSnapshot, family: &str) -> Value {
    match family {
        "collaboration" => collaboration_snapshot_summary(snapshot),
        "data" => data_snapshot_summary(snapshot),
        "diagram" => diagram_snapshot_summary(snapshot),
        "flow_element_metadata" => flow_element_metadata_snapshot_summary(snapshot),
        "resource_role" => resource_role_snapshot_summary(snapshot),
        _ => json!({ "root": root_snapshot_summary(snapshot) }),
    }
}

pub(super) fn root_snapshot_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    json!({
        "definitions_id": snapshot.root.definitions_id,
        "model_namespace_uri": snapshot.root.model_namespace_uri,
        "import_count": snapshot.root.import_count,
        "extension_count": snapshot.root.extension_count,
        "relationship_count": snapshot.root.relationship_count,
        "diagram_count": snapshot.root.diagram_count,
        "collaboration_count": snapshot.root.collaboration_count,
        "process_count": snapshot.root.process_count,
        "item_definition_count": snapshot.root.item_definition_count,
        "message_count": snapshot.root.message_count,
        "interface_count": snapshot.root.interface_count,
        "end_point_count": snapshot.root.end_point_count,
        "resource_count": snapshot.root.resource_count,
        "category_count": snapshot.root.category_count,
        "correlation_property_count": snapshot.root.correlation_property_count,
        "error_count": snapshot.root.error_count,
        "escalation_count": snapshot.root.escalation_count,
        "signal_count": snapshot.root.signal_count,
        "data_store_count": snapshot.root.data_store_count,
        "partner_entity_count": snapshot.root.partner_entity_count,
        "partner_role_count": snapshot.root.partner_role_count,
        "global_task_count": snapshot.root.global_task_count,
    })
}

pub(super) fn diagram_snapshot_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    let diagrams = snapshot
        .root
        .diagrams
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|diagram| {
            let shape_count = diagram.plane.as_ref().map_or(0, |plane| plane.shapes.len());
            let edge_count = diagram.plane.as_ref().map_or(0, |plane| plane.edges.len());
            json!({
                "diagram_id": diagram.diagram_id,
                "name": diagram.name,
                "resolution": diagram.resolution,
                "label_style_count": diagram.label_styles.len(),
                "shape_count": shape_count,
                "edge_count": edge_count,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "root": root_snapshot_summary(snapshot),
        "diagram_count": snapshot.root.diagram_count,
        "diagrams_truncated": snapshot.root.diagrams.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "diagrams": diagrams,
    })
}

pub(super) fn resource_role_snapshot_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    json!({
        "root": root_snapshot_summary(snapshot),
        "resource_roles": resource_role_summary(snapshot),
    })
}

pub(super) fn flow_element_metadata_snapshot_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    json!({
        "root": root_snapshot_summary(snapshot),
        "flow_element_metadata": flow_element_metadata_summary(snapshot),
    })
}

pub(super) fn collaboration_snapshot_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    let counts = collaboration_counts(snapshot);
    let collaborations = snapshot
        .collaborations
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(collaboration_evidence)
        .collect::<Vec<_>>();
    let item_definitions = item_definition_evidence(snapshot);
    let messages = message_evidence(snapshot);
    let interfaces = interface_evidence(snapshot);
    let partner_entities = partner_entity_evidence(snapshot);
    let partner_roles = partner_role_evidence(snapshot);
    let correlation_properties = correlation_property_evidence(snapshot);
    let process_callable = process_callable_summary(snapshot);
    let resource_roles = resource_role_summary(snapshot);
    let flow_element_metadata = flow_element_metadata_summary(snapshot);

    let mut summary = json!({
        "root": root_snapshot_summary(snapshot),
        "collaboration_count": snapshot.collaborations.len(),
        "partner_entity_count": snapshot.root.partner_entity_count,
        "partner_role_count": snapshot.root.partner_role_count,
        "end_point_count": snapshot.root.end_point_count,
        "participant_count": counts.participant,
        "participant_interface_ref_count": counts.participant_interface_ref,
        "participant_end_point_ref_count": counts.participant_end_point_ref,
        "participant_multiplicity_count": counts.participant_multiplicity,
        "message_flow_count": counts.message_flow,
        "conversation_node_count": counts.conversation_node,
        "conversation_link_count": counts.conversation_link,
        "conversation_association_count": counts.conversation_association,
        "participant_association_count": counts.participant_association,
        "message_flow_association_count": counts.message_flow_association,
        "correlation_key_count": counts.correlation_key,
        "choreography_activity_count": counts.choreography_activity,
        "artifact_association_count": counts.association,
        "artifact_group_count": counts.group,
        "text_annotation_count": counts.text_annotation,
        "item_definition_count": snapshot.root.item_definition_count,
        "message_count": snapshot.root.message_count,
        "interface_count": snapshot.root.interface_count,
        "correlation_property_count": snapshot.root.correlation_property_count,
        "partner_entities_truncated": snapshot.root.partner_entities.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "partner_roles_truncated": snapshot.root.partner_roles.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "collaborations_truncated": snapshot.collaborations.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "item_definitions_truncated": snapshot.root.item_definitions.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "messages_truncated": snapshot.root.messages.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "interfaces_truncated": snapshot.root.interfaces.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "correlation_properties_truncated": snapshot.root.correlation_properties.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "process_callable": process_callable,
        "resource_roles": resource_roles,
        "flow_element_metadata": flow_element_metadata,
        "item_definitions": item_definitions,
        "messages": messages,
        "interfaces": interfaces,
        "partner_entities": partner_entities,
        "partner_roles": partner_roles,
        "correlation_properties": correlation_properties,
        "collaborations": collaborations,
    });
    if let Some(summary_object) = summary.as_object_mut() {
        summary_object.insert("routing_boundary".to_string(), routing_boundary_evidence());
    }
    summary
}
