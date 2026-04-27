use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::BpmnEngineError;
use crate::lint_api::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde_json::json;
use std::borrow::Cow;
use std::ops::Range;

pub(super) fn human_task_standard_issues(source: &BpmnSourceFile) -> Vec<LintIssue> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);
    let mut state = HumanTaskStandardScanState::default();
    let mut issues = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                state.handle_start(source, &reader, &event, &mut issues, false);
            }
            Ok(Event::Empty(event)) => {
                state.handle_start(source, &reader, &event, &mut issues, true);
            }
            Ok(Event::End(event)) => state.handle_end(&event),
            Ok(Event::Eof) | Err(_) => return issues,
            Ok(_) => {}
        }
    }
}

pub(super) fn issue_from_bpmn_human_task_standard_error(
    source: &BpmnSourceFile,
    error: &BpmnEngineError,
) -> Option<LintIssue> {
    let BpmnEngineError::UnsupportedElement { element, .. } = error else {
        return None;
    };
    if !matches!(
        element.as_str(),
        "rendering" | "performer" | "resourceRole" | "participantRef" | "resourceParameterBinding"
    ) {
        return None;
    }
    human_task_standard_issues(source)
        .into_iter()
        .find(|issue| issue.evidence["element"].as_str() == Some(element.as_str()))
}

#[derive(Default)]
struct HumanTaskStandardScanState {
    active_tasks: Vec<HumanTaskContext>,
    active_roles: Vec<String>,
}

impl HumanTaskStandardScanState {
    fn handle_start(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        issues: &mut Vec<LintIssue>,
        is_empty: bool,
    ) {
        let tag = local_name(event.name().as_ref()).to_string();

        if is_human_interaction_task(&tag) {
            let context = HumanTaskContext {
                task_id: attribute_value(reader, event, "id"),
                task_kind: tag.clone(),
            };
            if !is_empty {
                self.active_tasks.push(context);
            }
            return;
        }

        if tag == "rendering" {
            if let Some(task) = self.active_rendering_task() {
                issues.push(unsupported_native_rendering_issue(
                    source, reader, event, task,
                ));
            }
            return;
        }

        if is_assignment_role(&tag) {
            if let Some(task) = self.active_tasks.last()
                && is_unsupported_assignment_role(&tag)
            {
                issues.push(unsupported_assignment_semantics_issue(
                    source, reader, event, task, &tag,
                ));
            }
            if !is_empty {
                self.active_roles.push(tag);
            }
            return;
        }

        if matches!(tag.as_str(), "resourceParameterBinding" | "participantRef")
            && let (Some(task), Some(role)) = (self.active_tasks.last(), self.active_roles.last())
        {
            issues.push(unsupported_assignment_child_issue(
                source, reader, event, task, role, &tag,
            ));
        }
    }

    fn handle_end(&mut self, event: &quick_xml::events::BytesEnd<'_>) {
        let name = event.name();
        let tag = local_name(name.as_ref());
        if is_human_interaction_task(tag) {
            self.active_tasks.pop();
        } else if is_assignment_role(tag) {
            self.active_roles.pop();
        }
    }

    fn active_rendering_task(&self) -> Option<&HumanTaskContext> {
        self.active_tasks
            .last()
            .filter(|task| matches!(task.task_kind.as_str(), "userTask" | "globalUserTask"))
    }
}

#[derive(Clone)]
struct HumanTaskContext {
    task_id: Option<String>,
    task_kind: String,
}

fn unsupported_native_rendering_issue(
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
        "OMG BPMN defines `rendering` as the native user-task rendering hook, but the current bounded Qianji runtime executes the typed `qianji:interaction` contract instead. Silent rendering fallback would make UI interpretation the runtime authority.",
        vec![
            "Model executable user interaction with one bounded `qianji:interaction` element on the `userTask`.".to_string(),
            "Preserve the standard `<bpmn:rendering>` intent as documentation only, or remove it from the executable slice until native rendering support is implemented.".to_string(),
            "Do not make downstream UI infer required fields, choices, or outputs from native rendering metadata in this bounded runtime.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by replacing runtime dependency on `<bpmn:rendering>` for user task '{task_id}' with a typed `qianji:interaction` contract. Preserve task id and workflow routing."
        ),
        json!({
            "source_id": source_id,
            "task_id": task.task_id.as_deref(),
            "task_kind": task.task_kind.as_str(),
            "element": "rendering",
            "supported_runtime_rendering_contract": "qianji:interaction",
        }),
    )
    .with_source_diagnostic(source_diagnostic(
        source,
        reader,
        event,
        "native BPMN rendering is not executable in this bounded slice",
        "Use `qianji:interaction` for executable form metadata, or keep native rendering as documentation only.",
    ))
    .with_structured_repair(json!({
        "schema_version": 1,
            "contract": "qianji.bpmn.human_task_interaction.v1",
            "strategy": "replace_native_rendering_with_qianji_interaction",
            "actions": [{
                "op": "add_or_use_qianji_interaction",
                "task_id": task.task_id.as_deref(),
                "allowed_interaction_types": ["input", "confirm", "choice", "choice_input"],
                "forbidden_runtime_dependency": "bpmn:rendering"
            }]
    }))
}

fn unsupported_assignment_semantics_issue(
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

fn unsupported_assignment_child_issue(
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

fn unsupported_assignment_issue(
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
            "contract": "qianji.bpmn.human_task_assignment.routing_metadata.v1",
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

fn source_diagnostic(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    label: impl Into<String>,
    help: impl Into<String>,
) -> LintSourceDiagnostic {
    let span = event_span(reader, event).unwrap_or(0..0);
    LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        label,
        help,
    )
}

fn event_span(reader: &Reader<&[u8]>, event: &BytesStart<'_>) -> Option<Range<usize>> {
    let event_end = usize::try_from(reader.buffer_position()).ok()?;
    let raw: &[u8] = event.as_ref();
    let start = event_end.checked_sub(raw.len() + 2)?;
    Some(start..event_end)
}

fn is_human_interaction_task(tag: &str) -> bool {
    matches!(
        tag,
        "userTask" | "manualTask" | "globalUserTask" | "globalManualTask"
    )
}

fn is_assignment_role(tag: &str) -> bool {
    matches!(
        tag,
        "humanPerformer" | "potentialOwner" | "performer" | "resourceRole"
    )
}

fn is_unsupported_assignment_role(tag: &str) -> bool {
    matches!(tag, "performer" | "resourceRole")
}

fn attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Option<String> {
    for attribute in event.attributes().flatten() {
        if local_name(attribute.key.as_ref()) != attribute_name {
            continue;
        }
        let value = attribute.decode_and_unescape_value(reader.decoder()).ok()?;
        return Some(match value {
            Cow::Borrowed(value) => value.to_string(),
            Cow::Owned(value) => value,
        });
    }
    None
}

fn local_name(raw: &[u8]) -> &str {
    std::str::from_utf8(raw)
        .ok()
        .map_or("", |name| name.rsplit(':').next().unwrap_or(name))
}
