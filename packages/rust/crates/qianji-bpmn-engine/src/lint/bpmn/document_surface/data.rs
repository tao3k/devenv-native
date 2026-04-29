use super::{
    BpmnDataAssociationExpressionSnapshot, BpmnDataAssociationSnapshot, BpmnDataStateSnapshot,
    BpmnDocumentSnapshot, BpmnInputSetSnapshot, BpmnOutputSetSnapshot, SNAPSHOT_EVIDENCE_LIMIT,
    Value, item_definition_evidence, json, root_snapshot_summary,
};

pub(super) fn data_snapshot_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
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
        "io_specification_count": io_specification_count,
        "data_input_association_count": data_input_association_count,
        "data_output_association_count": data_output_association_count,
        "process_data_truncated": snapshot.processes.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "process_data": process_data_evidence(snapshot),
    })
}

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

pub(super) fn data_association_evidence(association: &BpmnDataAssociationSnapshot) -> Value {
    json!({
        "association_id": association.association_id,
        "source_refs": association.source_refs,
        "target_ref": association.target_ref,
        "transformation": data_association_expression_evidence(association.transformation.as_ref()),
        "assignment_count": association.assignments.len(),
        "assignments": association.assignments.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|assignment| {
            json!({
                "assignment_id": assignment.assignment_id,
                "from": data_association_expression_evidence(assignment.from.as_ref()),
                "to": data_association_expression_evidence(assignment.to.as_ref()),
            })
        }).collect::<Vec<_>>(),
        "assignments_truncated": association.assignments.len() > SNAPSHOT_EVIDENCE_LIMIT,
    })
}

pub(super) fn input_set_evidence(input_set: &BpmnInputSetSnapshot) -> Value {
    json!({
        "set_id": input_set.set_id,
        "name": input_set.name,
        "data_input_refs": input_set.data_input_refs,
        "optional_input_refs": input_set.optional_input_refs,
        "while_executing_input_refs": input_set.while_executing_input_refs,
        "output_set_refs": input_set.output_set_refs,
    })
}

pub(super) fn output_set_evidence(output_set: &BpmnOutputSetSnapshot) -> Value {
    json!({
        "set_id": output_set.set_id,
        "name": output_set.name,
        "data_output_refs": output_set.data_output_refs,
        "optional_output_refs": output_set.optional_output_refs,
        "while_executing_output_refs": output_set.while_executing_output_refs,
        "input_set_refs": output_set.input_set_refs,
    })
}

pub(super) fn data_association_expression_evidence(
    expression: Option<&BpmnDataAssociationExpressionSnapshot>,
) -> Value {
    expression.map_or(Value::Null, |expression| {
        json!({
            "expression_id": expression.expression_id,
            "body": expression.body,
            "language": expression.language,
            "evaluates_to_type_ref": expression.evaluates_to_type_ref,
        })
    })
}

pub(super) fn data_state_evidence(state: Option<&BpmnDataStateSnapshot>) -> Value {
    state.map_or(Value::Null, |state| {
        json!({
            "data_state_id": state.data_state_id,
            "name": state.name,
        })
    })
}
