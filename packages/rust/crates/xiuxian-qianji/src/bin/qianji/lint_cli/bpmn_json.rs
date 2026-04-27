use std::io;
use std::path::Path;

use qianji_bpmn_engine::{
    BpmnParseOptions, BpmnSourceFile, LintReport, parse_bpmn_package,
    parse_gateway_condition_summary,
};
use quick_xml::Reader;
use quick_xml::escape::{resolve_predefined_entity, unescape};
use quick_xml::events::{BytesRef, BytesStart, Event};

use super::command::LintCliOutput;
use super::render::{issue_repair_plans, lint_domain_name};
use crate::json_output::{CliJsonEnvelope, render_cli_json};

pub(super) fn render_bpmn_lint_json_output(
    report: &LintReport,
    resolved_path: &Path,
    contents: &str,
) -> io::Result<LintCliOutput> {
    let exit_code = if report.ok { 0 } else { 2 };
    let rendered = render_cli_json(CliJsonEnvelope {
        kind: "qianji_lint_report",
        command: "lint",
        domain: lint_domain_name(&report.domain),
        path: resolved_path,
        source_id: &report.source_id,
        ok: report.ok,
        exit_code,
        report,
        analysis: Some(serde_json::json!({
            "gateway_conditions": collect_gateway_condition_analysis(contents),
            "repair_plans": issue_repair_plans(report),
        })),
    })?;
    Ok(LintCliOutput {
        rendered,
        exit_code,
    })
}

fn collect_gateway_condition_analysis(contents: &str) -> serde_json::Value {
    let source = BpmnSourceFile::new("<lint-json-analysis>", contents.to_string());
    let Ok(package) = parse_bpmn_package(&[source], &BpmnParseOptions::default()) else {
        return serde_json::Value::Array(collect_gateway_conditions_from_source(contents));
    };
    let mut conditions = Vec::new();
    for process in package.processes {
        for edge in process.edges {
            let Some(raw) = edge.condition_expression.as_deref() else {
                continue;
            };
            let source_node = process
                .nodes
                .get(edge.from as usize)
                .map_or_else(|| edge.from.to_string(), |node| node.bpmn_id.to_string());
            let target_node = process
                .nodes
                .get(edge.to as usize)
                .map_or_else(|| edge.to.to_string(), |node| node.bpmn_id.to_string());
            let parsed = parse_gateway_condition_summary(raw);
            let supported = parsed.is_some();
            conditions.push(serde_json::json!({
                "process_id": process.key.process_id,
                "source_ref": source_node,
                "target_ref": target_node,
                "raw": raw,
                "parsed": parsed,
                "supported": supported,
            }));
        }
    }
    serde_json::Value::Array(conditions)
}

struct SourceGatewayCondition {
    process_id: String,
    source_ref: String,
    target_ref: String,
    raw: String,
}

struct ActiveSequenceFlow {
    process_id: String,
    source_ref: String,
    target_ref: String,
    condition: String,
}

fn collect_gateway_conditions_from_source(contents: &str) -> Vec<serde_json::Value> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut element_stack: Vec<String> = Vec::new();
    let mut process_stack: Vec<String> = Vec::new();
    let mut active_flow: Option<ActiveSequenceFlow> = None;
    let mut condition_depth: Option<usize> = None;
    let mut conditions = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let tag = local_name(event.name().as_ref()).to_string();
                let next_depth = element_stack.len() + 1;
                if tag == "process" {
                    process_stack.push(attribute_value(&reader, &event, "id").unwrap_or_default());
                } else if tag == "sequenceFlow" {
                    active_flow = Some(ActiveSequenceFlow {
                        process_id: process_stack.last().cloned().unwrap_or_default(),
                        source_ref: attribute_value(&reader, &event, "sourceRef")
                            .unwrap_or_default(),
                        target_ref: attribute_value(&reader, &event, "targetRef")
                            .unwrap_or_default(),
                        condition: String::new(),
                    });
                } else if tag == "conditionExpression" && active_flow.is_some() {
                    condition_depth = Some(next_depth);
                }
                element_stack.push(tag);
            }
            Ok(Event::End(event)) => {
                let tag = local_name(event.name().as_ref()).to_string();
                if tag == "conditionExpression" && condition_depth == Some(element_stack.len()) {
                    condition_depth = None;
                }
                if tag == "sequenceFlow"
                    && let Some(flow) = active_flow.take()
                    && !flow.condition.trim().is_empty()
                {
                    conditions.push(render_source_gateway_condition(&SourceGatewayCondition {
                        process_id: flow.process_id,
                        source_ref: flow.source_ref,
                        target_ref: flow.target_ref,
                        raw: flow.condition.trim().to_string(),
                    }));
                }
                if tag == "process" {
                    process_stack.pop();
                }
                if element_stack
                    .last()
                    .is_some_and(|open_tag| open_tag == &tag)
                {
                    element_stack.pop();
                }
            }
            Ok(Event::Text(event)) if condition_depth.is_some() => {
                if let Some(flow) = active_flow.as_mut()
                    && let Ok(decoded) = event.decode()
                    && let Ok(text) = unescape(decoded.as_ref())
                {
                    flow.condition.push_str(text.as_ref());
                }
            }
            Ok(Event::CData(event)) if condition_depth.is_some() => {
                if let Some(flow) = active_flow.as_mut()
                    && let Ok(text) = event.decode()
                {
                    flow.condition.push_str(text.as_ref());
                }
            }
            Ok(Event::GeneralRef(event)) if condition_depth.is_some() => {
                if let Some(flow) = active_flow.as_mut()
                    && let Some(text) = reference_content(&event)
                {
                    flow.condition.push_str(&text);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }

    conditions
}

fn render_source_gateway_condition(condition: &SourceGatewayCondition) -> serde_json::Value {
    let parsed = parse_gateway_condition_summary(&condition.raw);
    let supported = parsed.is_some();
    serde_json::json!({
        "process_id": condition.process_id.as_str(),
        "source_ref": condition.source_ref.as_str(),
        "target_ref": condition.target_ref.as_str(),
        "raw": condition.raw.as_str(),
        "parsed": parsed,
        "supported": supported,
    })
}

fn attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Option<String> {
    event
        .attributes()
        .flatten()
        .find(|attribute| local_name(attribute.key.as_ref()) == attribute_name)
        .and_then(|attribute| {
            attribute
                .decode_and_unescape_value(reader.decoder())
                .ok()
                .map(std::borrow::Cow::into_owned)
        })
}

fn reference_content(reference: &BytesRef<'_>) -> Option<String> {
    if let Ok(Some(ch)) = reference.resolve_char_ref() {
        return Some(ch.to_string());
    }
    let decoded = reference.decode().ok()?;
    resolve_predefined_entity(decoded.as_ref()).map(ToOwned::to_owned)
}

fn local_name(name: &[u8]) -> &str {
    std::str::from_utf8(name)
        .ok()
        .map_or("", |raw| raw.rsplit(':').next().unwrap_or(raw))
}
