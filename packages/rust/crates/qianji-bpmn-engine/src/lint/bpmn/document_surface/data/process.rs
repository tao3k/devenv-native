use crate::bpmn_model_api::BpmnDocumentSnapshot;
use crate::lint::bpmn::document_surface::shared::SNAPSHOT_EVIDENCE_LIMIT;
use serde_json::{Value, json};

use super::association::data_association_evidence;
use super::set::{input_set_evidence, output_set_evidence};
use super::state::data_state_evidence;

pub(super) fn process_data_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .processes
        .iter()
        .filter(|process| {
            process.data_object_count
                + process.data_object_reference_count
                + process.data_store_reference_count
                + process.io_specification_count
                + process.data_input_association_count
                + process.data_output_association_count
                > 0
        })
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|process| {
            json!({
                "process_id": process.process_id,
                "data_object_count": process.data_object_count,
                "data_object_reference_count": process.data_object_reference_count,
                "data_store_reference_count": process.data_store_reference_count,
                "io_specification_count": process.io_specification_count,
                "data_input_association_count": process.data_input_association_count,
                "data_output_association_count": process.data_output_association_count,
                "data_objects": process.data_objects.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|object| {
                    json!({
                        "data_object_id": object.data_object_id,
                        "name": object.name,
                        "item_subject_ref": object.item_subject_ref,
                        "is_collection": object.is_collection,
                        "data_state": data_state_evidence(object.data_state.as_ref()),
                    })
                }).collect::<Vec<_>>(),
                "data_object_references": process.data_object_references.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|reference| {
                    json!({
                        "data_object_reference_id": reference.data_object_reference_id,
                        "name": reference.name,
                        "data_object_ref": reference.data_object_ref,
                        "item_subject_ref": reference.item_subject_ref,
                        "data_state": data_state_evidence(reference.data_state.as_ref()),
                    })
                }).collect::<Vec<_>>(),
                "data_store_references": process.data_store_references.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|reference| {
                    json!({
                        "data_store_reference_id": reference.data_store_reference_id,
                        "name": reference.name,
                        "data_store_ref": reference.data_store_ref,
                        "item_subject_ref": reference.item_subject_ref,
                        "data_state": data_state_evidence(reference.data_state.as_ref()),
                    })
                }).collect::<Vec<_>>(),
                "io_specifications": process.io_specifications.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|spec| {
                    json!({
                        "io_specification_id": spec.io_specification_id,
                        "data_inputs": spec.data_inputs.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|input| {
                            json!({
                                "data_id": input.data_id,
                                "name": input.name,
                                "item_subject_ref": input.item_subject_ref,
                                "is_collection": input.is_collection,
                                "data_state": data_state_evidence(input.data_state.as_ref()),
                            })
                        }).collect::<Vec<_>>(),
                        "data_outputs": spec.data_outputs.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|output| {
                            json!({
                                "data_id": output.data_id,
                                "name": output.name,
                                "item_subject_ref": output.item_subject_ref,
                                "is_collection": output.is_collection,
                                "data_state": data_state_evidence(output.data_state.as_ref()),
                            })
                        }).collect::<Vec<_>>(),
                        "input_sets": spec.input_sets.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(input_set_evidence).collect::<Vec<_>>(),
                        "output_sets": spec.output_sets.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(output_set_evidence).collect::<Vec<_>>(),
                        "input_sets_truncated": spec.input_sets.len() > SNAPSHOT_EVIDENCE_LIMIT,
                        "output_sets_truncated": spec.output_sets.len() > SNAPSHOT_EVIDENCE_LIMIT,
                    })
                }).collect::<Vec<_>>(),
                "data_input_associations": process.data_input_associations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|association| {
                    data_association_evidence(association)
                }).collect::<Vec<_>>(),
                "data_output_associations": process.data_output_associations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|association| {
                    data_association_evidence(association)
                }).collect::<Vec<_>>(),
            })
        })
        .collect()
}
