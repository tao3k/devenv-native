use crate::bpmn_model_api::{
    BpmnDataAssociationSnapshot, BpmnDataStoreReferenceSnapshot, BpmnDocumentSnapshot,
    BpmnProcessSnapshot,
};
use serde_json::{Value, json};

pub(super) fn data_store_binding_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    let mut bindings = Vec::new();
    for process in &snapshot.processes {
        bindings.extend(data_store_input_binding_evidence(process));
        bindings.extend(data_store_output_binding_evidence(process));
    }
    bindings
}

fn data_store_input_binding_evidence(process: &BpmnProcessSnapshot) -> Vec<Value> {
    process
        .data_input_associations
        .iter()
        .flat_map(|association| {
            association.source_refs.iter().filter_map(|source_ref| {
                data_store_reference_by_id(process, source_ref).map(|reference| {
                    data_store_binding_value(
                        process,
                        "dataInputAssociation",
                        association,
                        "sourceRef",
                        source_ref,
                        reference,
                    )
                })
            })
        })
        .collect()
}

fn data_store_output_binding_evidence(process: &BpmnProcessSnapshot) -> Vec<Value> {
    process
        .data_output_associations
        .iter()
        .filter_map(|association| {
            let target_ref = association.target_ref.as_deref()?;
            data_store_reference_by_id(process, target_ref).map(|reference| {
                data_store_binding_value(
                    process,
                    "dataOutputAssociation",
                    association,
                    "targetRef",
                    target_ref,
                    reference,
                )
            })
        })
        .collect()
}

fn data_store_reference_by_id<'a>(
    process: &'a BpmnProcessSnapshot,
    reference_id: &str,
) -> Option<&'a BpmnDataStoreReferenceSnapshot> {
    process
        .data_store_references
        .iter()
        .find(|reference| reference.data_store_reference_id.as_deref() == Some(reference_id))
}

fn data_store_binding_value(
    process: &BpmnProcessSnapshot,
    association_kind: &str,
    association: &BpmnDataAssociationSnapshot,
    usage: &str,
    reference_id: &str,
    reference: &BpmnDataStoreReferenceSnapshot,
) -> Value {
    json!({
        "process_id": process.process_id,
        "association_kind": association_kind,
        "association_id": association.association_id,
        "usage": usage,
        "reference_id": reference_id,
        "data_store_reference_id": reference.data_store_reference_id,
        "data_store_ref": reference.data_store_ref,
        "name": reference.name,
    })
}
