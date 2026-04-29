use super::{
    BpmnDocumentSnapshot, BpmnGlobalTaskSnapshot, BpmnResourceRoleSnapshot, ResourceRoleCounts,
    SNAPSHOT_EVIDENCE_LIMIT, Value, json,
};

pub(in crate::lint::bpmn::document_surface) fn resource_role_counts(
    snapshot: &BpmnDocumentSnapshot,
) -> ResourceRoleCounts {
    let mut counts = ResourceRoleCounts::default();
    for process in &snapshot.processes {
        counts.process_role += process.resource_role_count;
        for role in &process.resource_roles {
            counts.parameter_binding += role.parameter_bindings.len();
            counts.assignment_expression += usize::from(role.assignment_expression.is_some());
        }
    }
    for task in &snapshot.root.global_tasks {
        counts.global_task_role += task.resource_role_count;
        for role in &task.resource_roles {
            counts.parameter_binding += role.parameter_bindings.len();
            counts.assignment_expression += usize::from(role.assignment_expression.is_some());
        }
    }
    counts
}

pub(in crate::lint::bpmn::document_surface) fn resource_role_summary(
    snapshot: &BpmnDocumentSnapshot,
) -> Value {
    let counts = resource_role_counts(snapshot);
    json!({
        "process_role_count": counts.process_role,
        "global_task_role_count": counts.global_task_role,
        "parameter_binding_count": counts.parameter_binding,
        "assignment_expression_count": counts.assignment_expression,
        "processes_truncated": snapshot.processes.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "global_tasks_truncated": snapshot.root.global_tasks.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "processes": process_resource_role_evidence(snapshot),
        "global_tasks": global_task_resource_role_evidence(snapshot),
    })
}

pub(in crate::lint::bpmn::document_surface) fn process_resource_role_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .processes
        .iter()
        .filter(|process| process.resource_role_count > 0)
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|process| {
            json!({
                "process_id": process.process_id,
                "resource_role_count": process.resource_role_count,
                "resource_roles": process.resource_roles.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(resource_role_evidence).collect::<Vec<_>>(),
            })
        })
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn global_task_resource_role_evidence(
    snapshot: &BpmnDocumentSnapshot,
) -> Vec<Value> {
    snapshot
        .root
        .global_tasks
        .iter()
        .filter(|task| task.resource_role_count > 0)
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(global_task_resource_role_item_evidence)
        .collect()
}

pub(in crate::lint::bpmn::document_surface) fn global_task_resource_role_item_evidence(
    task: &BpmnGlobalTaskSnapshot,
) -> Value {
    json!({
        "task_kind": task.task_kind,
        "task_id": task.task_id,
        "resource_role_count": task.resource_role_count,
        "resource_roles": task.resource_roles.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(resource_role_evidence).collect::<Vec<_>>(),
    })
}

pub(in crate::lint::bpmn::document_surface) fn resource_role_evidence(
    role: &BpmnResourceRoleSnapshot,
) -> Value {
    json!({
        "role_kind": role.role_kind,
        "role_id": role.role_id,
        "name": role.name,
        "resource_ref": role.resource_ref,
        "assignment_expression_id": role.assignment_expression_id,
        "assignment_expression": role.assignment_expression,
        "assignment_expression_language": role.assignment_expression_language,
        "assignment_expression_evaluates_to_type_ref": role.assignment_expression_evaluates_to_type_ref,
        "parameter_binding_count": role.parameter_bindings.len(),
        "parameter_bindings": role.parameter_bindings.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|binding| {
            json!({
                "binding_id": binding.binding_id,
                "parameter_ref": binding.parameter_ref,
                "expression": binding.expression,
                "expression_language": binding.expression_language,
                "expression_evaluates_to_type_ref": binding.expression_evaluates_to_type_ref,
            })
        }).collect::<Vec<_>>(),
    })
}
