use super::{
    BpmnDocumentSnapshot, BpmnGlobalTaskSnapshot, BpmnIoBindingSnapshot, BpmnProcessSnapshot,
    ProcessCallableCounts, SNAPSHOT_EVIDENCE_LIMIT, Value, json,
};

pub(in crate::lint::bpmn::document_surface) fn process_callable_counts(
    snapshot: &BpmnDocumentSnapshot,
) -> ProcessCallableCounts {
    let mut counts =
        snapshot
            .processes
            .iter()
            .fold(ProcessCallableCounts::default(), |mut counts, process| {
                counts.support += process.support_count;
                counts.property += process.property_count;
                counts.correlation_subscription += process.correlation_subscription_count;
                counts.correlation_binding += process
                    .correlation_subscriptions
                    .iter()
                    .map(|subscription| subscription.bindings.len())
                    .sum::<usize>();
                counts.process_io_binding += process.io_binding_count;
                counts
            });
    counts.global_task_io_specification = snapshot
        .root
        .global_tasks
        .iter()
        .map(|task| task.io_specification_count)
        .sum();
    counts.global_task_io_binding = snapshot
        .root
        .global_tasks
        .iter()
        .map(|task| task.io_binding_count)
        .sum();
    counts
}

pub(in crate::lint::bpmn::document_surface) fn process_callable_summary(
    snapshot: &BpmnDocumentSnapshot,
) -> Value {
    let counts = process_callable_counts(snapshot);
    json!({
        "support_count": counts.support,
        "property_count": counts.property,
        "correlation_subscription_count": counts.correlation_subscription,
        "correlation_binding_count": counts.correlation_binding,
        "process_io_binding_count": counts.process_io_binding,
        "global_task_io_specification_count": counts.global_task_io_specification,
        "global_task_io_binding_count": counts.global_task_io_binding,
        "metadata_truncated": snapshot.processes.len() > SNAPSHOT_EVIDENCE_LIMIT || snapshot.root.global_tasks.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "processes": process_callable_metadata_evidence(snapshot),
        "global_tasks": global_task_callable_metadata_evidence(snapshot),
    })
}

pub(in crate::lint::bpmn::document_surface) fn process_callable_metadata_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .processes
        .iter()
        .filter(|process| has_process_callable_metadata(process))
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|process| {
            json!({
                "process_id": process.process_id,
                "process_type": process.process_type,
                "is_closed": process.is_closed,
                "is_executable": process.is_executable,
                "definitional_collaboration_ref": process.definitional_collaboration_ref,
                "support_count": process.support_count,
                "supports": process.supports,
                "property_count": process.property_count,
                "properties": process.properties.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|property| {
                    json!({
                        "property_id": property.property_id,
                        "name": property.name,
                        "item_subject_ref": property.item_subject_ref,
                    })
                }).collect::<Vec<_>>(),
                "correlation_subscription_count": process.correlation_subscription_count,
                "correlation_subscriptions": process.correlation_subscriptions.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|subscription| {
                    json!({
                        "subscription_id": subscription.subscription_id,
                        "correlation_key_ref": subscription.correlation_key_ref,
                        "binding_count": subscription.bindings.len(),
                        "bindings": subscription.bindings.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|binding| {
                            json!({
                                "binding_id": binding.binding_id,
                                "correlation_property_ref": binding.correlation_property_ref,
                                "data_path": binding.data_path,
                                "data_path_language": binding.data_path_language,
                                "data_path_evaluates_to_type_ref": binding.data_path_evaluates_to_type_ref,
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "io_binding_count": process.io_binding_count,
                "io_bindings": process.io_bindings.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(io_binding_evidence).collect::<Vec<_>>(),
            })
        })
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn global_task_callable_metadata_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .root
        .global_tasks
        .iter()
        .filter(|task| has_global_task_callable_metadata(task))
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|task| {
            json!({
                "task_kind": task.task_kind,
                "task_id": task.task_id,
                "name": task.name,
                "supported_interface_refs": task.supported_interface_refs,
                "io_specification_count": task.io_specification_count,
                "io_specifications": task.io_specifications.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|spec| {
                    json!({
                        "io_specification_id": spec.io_specification_id,
                        "data_input_count": spec.data_inputs.len(),
                        "data_inputs": spec.data_inputs.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|input| {
                            json!({
                                "data_id": input.data_id,
                                "name": input.name,
                                "item_subject_ref": input.item_subject_ref,
                                "is_collection": input.is_collection,
                            })
                        }).collect::<Vec<_>>(),
                        "data_output_count": spec.data_outputs.len(),
                        "data_outputs": spec.data_outputs.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|output| {
                            json!({
                                "data_id": output.data_id,
                                "name": output.name,
                                "item_subject_ref": output.item_subject_ref,
                                "is_collection": output.is_collection,
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "io_binding_count": task.io_binding_count,
                "io_bindings": task.io_bindings.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(io_binding_evidence).collect::<Vec<_>>(),
            })
        })
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn io_binding_evidence(
    binding: &BpmnIoBindingSnapshot,
) -> Value {
    json!({
        "binding_id": binding.binding_id,
        "operation_ref": binding.operation_ref,
        "input_data_ref": binding.input_data_ref,
        "output_data_ref": binding.output_data_ref,
    })
}

pub(in crate::lint::bpmn::document_surface) fn has_process_callable_metadata(
    process: &BpmnProcessSnapshot,
) -> bool {
    process.process_type.is_some()
        || process.is_closed.is_some()
        || process.definitional_collaboration_ref.is_some()
        || process.support_count > 0
        || process.property_count > 0
        || process.correlation_subscription_count > 0
        || process.io_binding_count > 0
}

pub(in crate::lint::bpmn::document_surface) fn has_global_task_callable_metadata(
    task: &BpmnGlobalTaskSnapshot,
) -> bool {
    !task.supported_interface_refs.is_empty()
        || task.io_specification_count > 0
        || task.io_binding_count > 0
}
