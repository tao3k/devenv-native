use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint_api::LintIssue;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde_json::json;
use std::borrow::Cow;

const SUPPORTED_INTERACTION_TYPES: &[&str] = &["input", "confirm", "choice", "choice_input"];

pub(super) fn qianji_extension_issue(source: &BpmnSourceFile) -> Option<LintIssue> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event)) => {
                if !is_qianji_interaction(&event) {
                    continue;
                }
                let interaction_type = attribute_value(&reader, &event, "type");
                match interaction_type.as_deref() {
                    Some(kind) if SUPPORTED_INTERACTION_TYPES.contains(&kind) => {}
                    Some(kind) => {
                        return Some(unsupported_interaction_type_issue(source, kind));
                    }
                    None => return Some(missing_interaction_type_issue(source)),
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn is_qianji_interaction(event: &BytesStart<'_>) -> bool {
    let name = event.name();
    let raw_name = std::str::from_utf8(name.as_ref()).unwrap_or_default();
    raw_name == "qianji:interaction"
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

fn unsupported_interaction_type_issue(
    source: &BpmnSourceFile,
    interaction_type: &str,
) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.unsupported_qianji_interaction_type",
        "Qianji user interaction type is unsupported",
        format!(
            "Source '{source_id}' uses qianji interaction type '{interaction_type}', which is outside the active qianji extension contract."
        ),
        "Qianji-owned user-task interaction rendering currently supports only a bounded native UI subset: input, confirm, choice, and choice_input.",
        vec![
            "Replace unsupported interaction types such as `free_form` with `input` for plain text input.".to_string(),
            "Use `choice_input` when the prompt needs option selection plus optional free-form feedback.".to_string(),
            "Keep the answer mapping in declared `qianji:outputs` so downstream gateways only consume declared variables.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by changing qianji interaction type '{interaction_type}' to one supported value: input, confirm, choice, or choice_input. Preserve the user prompt, choices, freeText fields, and declared qianji outputs."
        ),
        json!({
            "source_id": source_id,
            "interaction_type": interaction_type,
            "supported_interaction_types": SUPPORTED_INTERACTION_TYPES,
        }),
    )
    .with_structured_repair(interaction_repair_plan(
        source_id,
        "replace_unsupported_interaction_type",
        json!({
            "op": "set_attribute",
            "element": "qianji:interaction",
            "attribute": "type",
            "allowed_values": SUPPORTED_INTERACTION_TYPES,
            "selection_hint": {
                "input": "plain free-form answer",
                "confirm": "yes/no approval",
                "choice": "bounded option selection",
                "choice_input": "option selection plus optional free-form feedback"
            },
            "replace": interaction_type
        }),
    ))
}

fn missing_interaction_type_issue(source: &BpmnSourceFile) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.missing_qianji_interaction_type",
        "Qianji user interaction type is missing",
        format!("Source '{source_id}' has qianji:interaction without a `type` attribute."),
        "Qianji-owned user-task interaction rendering needs an explicit bounded native UI type.",
        vec![
            "Add `type=\"input\"` for plain free-form text input.".to_string(),
            "Add `type=\"confirm\"`, `type=\"choice\"`, or `type=\"choice_input\"` for bounded approval and selection checkpoints.".to_string(),
            "Keep the selected type aligned with the declared `qianji:outputs` mapping.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by adding one supported qianji interaction type: input, confirm, choice, or choice_input."
        ),
        json!({
            "source_id": source_id,
            "supported_interaction_types": SUPPORTED_INTERACTION_TYPES,
        }),
    )
    .with_structured_repair(interaction_repair_plan(
        source_id,
        "add_missing_interaction_type",
        json!({
            "op": "set_attribute",
            "element": "qianji:interaction",
            "attribute": "type",
            "allowed_values": SUPPORTED_INTERACTION_TYPES,
            "default_when_unsure": "choice_input"
        }),
    ))
}

fn interaction_repair_plan(
    source_id: &str,
    strategy: &'static str,
    action: serde_json::Value,
) -> serde_json::Value {
    let actions = serde_json::Value::Array(vec![action]);
    json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.user_task.interaction.v1",
        "strategy": strategy,
        "target": {
            "source_id": source_id,
        },
        "construct_cards": ["user-task.interaction"],
        "actions": actions,
    })
}
