use super::{
    BpmnDocumentSnapshot, BpmnFlowElementMetadataSnapshot, FlowElementMetadataCounts,
    SNAPSHOT_EVIDENCE_LIMIT, Value, json,
};

pub(in crate::lint::bpmn::document_surface) fn flow_element_metadata_counts(
    snapshot: &BpmnDocumentSnapshot,
) -> FlowElementMetadataCounts {
    let mut counts = FlowElementMetadataCounts::default();
    for process in &snapshot.processes {
        counts.element += process.flow_element_metadata_count;
        for metadata in &process.flow_element_metadata {
            counts.auditing += usize::from(metadata.has_auditing);
            counts.monitoring += usize::from(metadata.has_monitoring);
            counts.category_value_ref += metadata.category_value_refs.len();
        }
    }
    counts
}

pub(in crate::lint::bpmn::document_surface) fn flow_element_metadata_summary(
    snapshot: &BpmnDocumentSnapshot,
) -> Value {
    let counts = flow_element_metadata_counts(snapshot);
    json!({
        "element_count": counts.element,
        "auditing_count": counts.auditing,
        "monitoring_count": counts.monitoring,
        "category_value_ref_count": counts.category_value_ref,
        "processes_truncated": snapshot.processes.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "processes": process_flow_element_metadata_evidence(snapshot),
    })
}

pub(in crate::lint::bpmn::document_surface) fn process_flow_element_metadata_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .processes
        .iter()
        .filter(|process| process.flow_element_metadata_count > 0)
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|process| {
            json!({
                "process_id": process.process_id,
                "flow_element_metadata_count": process.flow_element_metadata_count,
                "flow_elements": process.flow_element_metadata.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(flow_element_metadata_evidence).collect::<Vec<_>>(),
            })
        })
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn flow_element_metadata_evidence(
    metadata: &BpmnFlowElementMetadataSnapshot,
) -> Value {
    json!({
        "element_kind": metadata.element_kind,
        "element_id": metadata.element_id,
        "name": metadata.name,
        "has_auditing": metadata.has_auditing,
        "auditing_id": metadata.auditing_id,
        "has_monitoring": metadata.has_monitoring,
        "monitoring_id": metadata.monitoring_id,
        "category_value_refs": metadata.category_value_refs,
    })
}
