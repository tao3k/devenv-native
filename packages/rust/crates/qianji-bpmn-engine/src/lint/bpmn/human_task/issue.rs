use super::{
    BpmnSourceFile, BytesStart, CallActivityContext, GlobalTaskContext, HumanTaskContext,
    LintIssue, Reader, json, source_diagnostic, source_diagnostic_from_span,
};

pub(super) fn unsupported_global_task_binding_issue(
    source: &BpmnSourceFile,
    call_activity: &CallActivityContext,
    task: &GlobalTaskContext,
) -> LintIssue {
    let source_id = &source.source_id;
    let process_id = call_activity.process_id.as_deref().unwrap_or("<unknown>");
    let activity_id = call_activity.activity_id.as_deref().unwrap_or("<unknown>");
    LintIssue::new(
        "bpmn.unsupported_global_task_binding",
        "Global task binding is not executable",
        format!(
            "Source '{source_id}' process '{process_id}' call activity '{activity_id}' targets global task '{}'.",
            task.task_id
        ),
        "OMG BPMN global tasks are reusable root definitions, but the bounded Rust runtime currently executes `callActivity` only when `calledElement` points to another executable process in the same BPMN package. Treating a global task as an ordinary process child would move the runtime binding decision out of Rust and into downstream adapter inference.",
        vec![
            "Model executable work as a process-local task with one bounded native BPMN IO contract.".to_string(),
            "If reusable behavior is required now, wrap the global task intent in an executable process and point `callActivity calledElement` at that process id.".to_string(),
            "Do not let adapters or UI code resolve a global task id into executable host dispatch behavior.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by changing call activity '{activity_id}' so `calledElement` targets an executable process id, or by replacing the call activity with a local task that carries a typed native BPMN IO contract."
        ),
        json!({
            "source_id": source_id,
            "process_id": call_activity.process_id.as_deref(),
            "call_activity_id": call_activity.activity_id.as_deref(),
            "called_element": call_activity.called_element.as_str(),
            "global_task_id": task.task_id.as_str(),
            "global_task_kind": task.task_kind.as_str(),
            "element": "callActivity",
            "supported_call_activity_target": "same-package executable process",
            "unsupported_binding": "global task",
        }),
    )
    .with_source_diagnostic(source_diagnostic_from_span(
        source,
        call_activity.span.clone(),
        "global task ids are not executable callActivity targets",
        "Point `calledElement` at an executable process, or model the work as a local task with native BPMN IO metadata.",
    ))
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "bpmn.native.global_task_policy.v1",
        "strategy": "replace_global_task_binding_with_rust_owned_executable_surface",
        "actions": [{
            "op": "replace_call_activity_target",
            "call_activity_id": call_activity.activity_id.as_deref(),
            "forbidden_called_element": call_activity.called_element.as_str(),
            "allowed_targets": ["same-package executable process"],
            "allowed_alternative": "local executable task with native BPMN IO metadata"
        }]
    }))
}

pub(super) fn native_rendering_issue(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    task: &HumanTaskContext,
) -> LintIssue {
    match task.task_kind.as_str() {
        "manualTask" | "globalManualTask" => {
            invalid_manual_task_rendering_issue(source, reader, event, task)
        }
        _ => unsupported_native_rendering_issue(source, reader, event, task),
    }
}

pub(super) fn unsupported_native_rendering_issue(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    task: &HumanTaskContext,
) -> LintIssue {
    let source_id = &source.source_id;
    let task_id = task.task_id.as_deref().unwrap_or("<unknown>");
    LintIssue::new(
        "bpmn.unsupported_human_task_rendering",
        "Native BPMN user-task rendering is deferred",
        format!(
            "Source '{source_id}' user task '{task_id}' declares standard BPMN `<rendering>` metadata."
        ),
        "OMG BPMN defines `rendering` as the native user-task rendering hook, but the current bounded runtime executes a typed native BPMN IO interaction contract instead. Silent rendering fallback would make UI interpretation the runtime authority.",
        vec![
            "Model executable user interaction with native BPMN `documentation`, `ioSpecification`, `dataInputAssociation`, and `dataOutputAssociation` metadata on the `userTask`.".to_string(),
            "Preserve the standard `<bpmn:rendering>` intent as documentation only, or remove it from the executable slice until native rendering support is implemented.".to_string(),
            "Do not make downstream UI infer required fields, choices, or outputs from native rendering metadata in this bounded runtime.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by replacing runtime dependency on `<bpmn:rendering>` for user task '{task_id}' with native BPMN IO interaction metadata. Preserve task id and workflow routing."
        ),
        json!({
            "source_id": source_id,
            "task_id": task.task_id.as_deref(),
            "task_kind": task.task_kind.as_str(),
            "element": "rendering",
            "supported_runtime_rendering_contract": "native_bpmn_io",
        }),
    )
    .with_source_diagnostic(source_diagnostic(
        source,
        reader,
        event,
        "native BPMN rendering is not executable in this bounded slice",
        "Use native BPMN IO metadata for executable form metadata, or keep native rendering as documentation only.",
    ))
    .with_structured_repair(json!({
        "schema_version": 1,
            "contract": "bpmn.native_human_task_io.v1",
            "strategy": "replace_native_rendering_with_native_io_interaction",
            "actions": [{
                "op": "add_or_use_native_bpmn_io_interaction",
                "task_id": task.task_id.as_deref(),
                "allowed_interaction_types": ["input", "confirm", "choice", "choice_input"],
                "forbidden_runtime_dependency": "bpmn:rendering"
            }]
    }))
}

pub(super) fn invalid_manual_task_rendering_issue(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    task: &HumanTaskContext,
) -> LintIssue {
    let source_id = &source.source_id;
    let task_id = task.task_id.as_deref().unwrap_or("<unknown>");
    LintIssue::new(
        "bpmn.invalid_manual_task_rendering",
        "Manual task rendering is not a BPMN execution contract",
        format!(
            "Source '{source_id}' manual task '{task_id}' declares standard BPMN `<rendering>` metadata."
        ),
        "OMG BPMN defines `rendering` under `userTask` and `globalUserTask`, not `manualTask` or `globalManualTask`. Qianji exposes manual tasks as host-visible pending work for operator acknowledgement, but it does not treat manual-task rendering metadata as executable UI.",
        vec![
            "If the activity needs a runtime-managed human form, model it as a `userTask` with native BPMN IO interaction metadata.".to_string(),
            "If the activity is truly external manual work, keep it as a `manualTask` and place any executable acknowledgement fields in native BPMN IO metadata, not standard `<bpmn:rendering>`.".to_string(),
            "Do not let downstream UI infer manual-task required fields, choices, or outputs from non-standard rendering metadata.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by removing `<bpmn:rendering>` from manual task '{task_id}'. Use a `userTask` for runtime-managed form rendering, or keep the manual task with native BPMN IO acknowledgement metadata."
        ),
        json!({
            "source_id": source_id,
            "task_id": task.task_id.as_deref(),
            "task_kind": task.task_kind.as_str(),
            "element": "rendering",
            "allowed_standard_rendering_tasks": ["userTask", "globalUserTask"],
            "supported_runtime_rendering_contract": "native_bpmn_io",
        }),
    )
    .with_source_diagnostic(source_diagnostic(
        source,
        reader,
        event,
        "manual tasks do not own standard BPMN rendering metadata",
        "Use `userTask` for runtime-managed forms, or keep manual acknowledgement metadata in native BPMN IO.",
    ))
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "bpmn.native_human_task_io.v1",
        "strategy": "remove_manual_task_rendering_or_model_user_task",
        "actions": [{
            "op": "remove_bpmn_rendering_from_manual_task",
            "task_id": task.task_id.as_deref(),
            "allowed_standard_rendering_tasks": ["userTask", "globalUserTask"],
            "allowed_executable_contract": "native_bpmn_io"
        }]
    }))
}

pub(super) fn unsupported_assignment_semantics_issue(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    task: &HumanTaskContext,
    element: &str,
) -> LintIssue {
    unsupported_assignment_issue(
        source,
        reader,
        event,
        task,
        element,
        "standard BPMN resource role is outside the current human-task routing metadata contract",
    )
}

pub(super) fn unsupported_assignment_child_issue(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    task: &HumanTaskContext,
    role: &str,
    element: &str,
) -> LintIssue {
    unsupported_assignment_issue(
        source,
        reader,
        event,
        task,
        element,
        &format!("`{element}` under `{role}` requires full BPMN resource assignment semantics"),
    )
}

pub(super) fn unsupported_assignment_issue(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    task: &HumanTaskContext,
    element: &str,
    label: &str,
) -> LintIssue {
    let source_id = &source.source_id;
    let task_id = task.task_id.as_deref().unwrap_or("<unknown>");
    LintIssue::new(
        "bpmn.unsupported_human_task_assignment_semantics",
        "Human-task assignment semantics exceed routing metadata",
        format!(
            "Source '{source_id}' human task '{task_id}' uses standard BPMN assignment element '<{element}>'."
        ),
        "Qianji currently preserves `humanPerformer` and `potentialOwner` names, `resourceRef`, and `resourceAssignmentExpression/formalExpression` text as routing metadata only. It does not resolve generic resource roles, participant refs, resource parameter bindings, claim, release, worklist, or authorization semantics.",
        vec![
            "Keep `humanPerformer` or `potentialOwner` with a simple `resourceRef` or `resourceAssignmentExpression/formalExpression` when a routing hint is enough.".to_string(),
            "Remove generic `performer`, `resourceRole`, `participantRef`, and `resourceParameterBinding` dependencies from the executable slice until full assignment semantics are implemented.".to_string(),
            "Do not enforce claim, authorization, or worklist behavior in downstream UI only; model it as a separate Rust-owned state transition surface when needed.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by reducing human task '{task_id}' assignment metadata to supported routing hints (`humanPerformer` or `potentialOwner` with simple `resourceRef` or `formalExpression`), or defer the full assignment/worklist behavior to a later Rust-owned contract."
        ),
        json!({
            "source_id": source_id,
            "task_id": task.task_id.as_deref(),
            "task_kind": task.task_kind.as_str(),
            "element": element,
            "supported_assignment_metadata": [
                "humanPerformer.name",
                "humanPerformer.resourceRef",
                "humanPerformer.resourceAssignmentExpression.formalExpression",
                "potentialOwner.name",
                "potentialOwner.resourceRef",
                "potentialOwner.resourceAssignmentExpression.formalExpression"
            ],
            "unsupported_semantics": [
                "generic performer/resourceRole resolution",
                "participantRef resolution",
                "resourceParameterBinding",
                "claim/release/worklist",
                "authorization"
            ],
        }),
    )
    .with_source_diagnostic(source_diagnostic(
        source,
        reader,
        event,
        label,
        "Keep only routing metadata now; implement full assignment as a separate Rust-owned contract later.",
    ))
    .with_structured_repair(json!({
        "schema_version": 1,
            "contract": "bpmn.native.human_task_assignment.routing_metadata.v1",
            "strategy": "reduce_full_assignment_to_routing_metadata",
            "actions": [{
                "op": "remove_or_defer_unsupported_assignment_semantics",
                "task_id": task.task_id.as_deref(),
                "element": element,
                "allowed_role_elements": ["humanPerformer", "potentialOwner"],
                "allowed_role_payloads": ["resourceRef", "resourceAssignmentExpression/formalExpression"]
        }]
    }))
}
