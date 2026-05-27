use crate::bpmn_model_api::BpmnDocumentSnapshot;
use crate::lint::bpmn::document_surface::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint::bpmn::document_surface::collaboration::item_definition_evidence;
use crate::lint::bpmn::document_surface::summary::root_snapshot_summary;
use serde_json::{Value, json};

use super::binding::data_store_binding_evidence;
use super::process::process_data_evidence;
use super::state::data_state_evidence;

pub(in crate::lint::bpmn::document_surface) fn data_snapshot_summary(
    snapshot: &BpmnDocumentSnapshot,
) -> Value {
    let data_store_bindings = data_store_binding_evidence(snapshot);
    let data_object_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_object_count)
        .sum::<usize>();
    let data_object_reference_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_object_reference_count)
        .sum::<usize>();
    let data_store_reference_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_store_reference_count)
        .sum::<usize>();
    let io_specification_count = snapshot
        .processes
        .iter()
        .map(|process| process.io_specification_count)
        .sum::<usize>();
    let data_input_association_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_input_association_count)
        .sum::<usize>();
    let data_output_association_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_output_association_count)
        .sum::<usize>();

    json!({
        "root": root_snapshot_summary(snapshot),
        "item_definition_count": snapshot.root.item_definition_count,
        "item_definitions": item_definition_evidence(snapshot),
        "data_object_count": data_object_count,
        "data_object_reference_count": data_object_reference_count,
        "data_store_count": snapshot.root.data_store_count,
        "data_stores": snapshot.root.data_stores.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|store| {
            json!({
                "data_store_id": store.data_store_id,
                "name": store.name,
                "item_subject_ref": store.item_subject_ref,
                "capacity": store.capacity,
                "is_unlimited": store.is_unlimited,
                "data_state": data_state_evidence(store.data_state.as_ref()),
            })
        }).collect::<Vec<_>>(),
        "data_store_reference_count": data_store_reference_count,
        "data_store_binding_count": data_store_bindings.len(),
        "data_store_bindings": data_store_bindings.iter().take(SNAPSHOT_EVIDENCE_LIMIT).cloned().collect::<Vec<_>>(),
        "data_store_bindings_truncated": data_store_bindings.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "io_specification_count": io_specification_count,
        "data_input_association_count": data_input_association_count,
        "data_output_association_count": data_output_association_count,
        "process_data_truncated": snapshot.processes.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "process_data": process_data_evidence(snapshot),
    })
}
