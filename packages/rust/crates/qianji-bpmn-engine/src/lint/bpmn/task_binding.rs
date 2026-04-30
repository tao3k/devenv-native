use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint_api::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde_json::json;
use std::borrow::Cow;

pub(super) fn task_operation_binding_issues(source: &BpmnSourceFile) -> Vec<LintIssue> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(false);
    let mut issues = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event))
                if is_operation_bound_task_element(&event) =>
            {
                let Some(operation_ref) = attribute_value(&reader, &event, "operationRef") else {
                    continue;
                };
                let task_kind = local_name(event.name().as_ref()).to_string();
                let task_id = attribute_value(&reader, &event, "id")
                    .unwrap_or_else(|| "<missing>".to_string());
                let span = reader_position(&reader)
                    .and_then(|event_end| start_event_span(event_end, &event));
                issues.push(operation_binding_issue(
                    source,
                    &task_kind,
                    &task_id,
                    &operation_ref,
                    span,
                ));
            }
            Ok(Event::Eof) | Err(_) => return issues,
            Ok(_) => {}
        }
    }
}

fn operation_binding_issue(
    source: &BpmnSourceFile,
    task_kind: &str,
    task_id: &str,
    operation_ref: &str,
    span: Option<std::ops::Range<usize>>,
) -> LintIssue {
    let source_id = &source.source_id;
    let issue = LintIssue::new(
        "bpmn.unsupported_operation_binding",
        "Task operation binding is deferred",
        format!(
            "Source '{source_id}' contains <{task_kind}> task '{task_id}' with operationRef '{operation_ref}'."
        ),
        "The bounded engine preserves standard BPMN interface and operation catalogs as metadata, but it does not resolve task-level `operationRef` into endpoint invocation, service binding, or external callable contract validation. Host dispatch remains driven by explicit task identity, bounded task Data/IO, message metadata where supported, and host-work request metadata.",
        vec![
            "Remove task-level `operationRef` from the executable task and keep the operation catalog as metadata until an invocation policy exists.".to_string(),
            "Represent executable request and response values with bounded task `ioSpecification`, `dataInputAssociation`, and `dataOutputAssociation` mappings.".to_string(),
            "If an external operation must run now, route it through the host-dispatched task payload instead of relying on BPMN interface-operation binding.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by removing operationRef '{operation_ref}' from <{task_kind}> task '{task_id}'. Preserve workflow intent, keep task Data/IO mappings explicit, and route any external call through host-dispatched task metadata rather than BPMN interface-operation invocation."
        ),
        json!({
            "source_id": source_id,
            "task_kind": task_kind,
            "task_id": task_id,
            "operation_ref": operation_ref,
            "bounded_surface": [
                "host_work_request_metadata",
                "task_io_inputs",
                "task_io_output_bindings",
                "messageRef_or_messageEventDefinition_for_send_receive_tasks"
            ],
            "metadata_only_surface": [
                "top_level_interface",
                "top_level_operation",
                "process_or_global_task_ioBinding"
            ],
            "deferred_semantics": [
                "interface_operation_invocation",
                "endpoint_binding",
                "external_callable_contract_validation"
            ]
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "bpmn.native.task.operation_binding_deferred.v1",
        "strategy": "remove_executable_operation_ref",
        "actions": [{
            "op": "remove_attribute",
            "element_kind": task_kind,
            "element_id": task_id,
            "attribute": "operationRef",
            "value": operation_ref
        }]
    }));

    if let Some(span) = span {
        issue.with_source_diagnostic(LintSourceDiagnostic::new(
            source_id,
            LintSourceSpan::new(span.start, span.end),
            "remove executable operationRef binding",
            "Keep interface/operation declarations as metadata and route execution through explicit host-work task IO.",
        ))
    } else {
        issue
    }
}

fn is_operation_bound_task_element(event: &BytesStart<'_>) -> bool {
    matches!(
        local_name(event.name().as_ref()),
        "serviceTask" | "sendTask" | "receiveTask"
    )
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

fn start_event_span(event_end: usize, event: &BytesStart<'_>) -> Option<std::ops::Range<usize>> {
    let raw: &[u8] = event.as_ref();
    let start = event_end.checked_sub(raw.len() + 2)?;
    Some(start..event_end)
}

fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
}

fn local_name(raw: &[u8]) -> &str {
    std::str::from_utf8(raw)
        .ok()
        .map_or("", |name| name.rsplit(':').next().unwrap_or(name))
}
