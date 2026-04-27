use crate::bpmn_parse_api::BpmnSourceFile;
use crate::ir_package_api::BpmnPackage;
use crate::lint_api::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use crate::repeat_condition::{
    GatewayConditionSummary, is_supported_gateway_condition, parse_gateway_condition_summary,
};
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashSet;

pub(super) fn ambiguous_boolean_gateway_condition_issues(
    source: &BpmnSourceFile,
    package: &BpmnPackage,
) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let interaction_outputs = collect_static_interaction_choice_outputs(&source.contents);
    for process in &package.processes {
        for edge in &process.edges {
            let Some(condition) = edge.condition_expression.as_deref() else {
                continue;
            };
            let Some(GatewayConditionSummary::BooleanPath { path, .. }) =
                parse_gateway_condition_summary(condition)
            else {
                continue;
            };
            let gateway_id = process
                .nodes
                .get(edge.from as usize)
                .map_or_else(|| edge.from.to_string(), |node| node.bpmn_id.to_string());
            if let Some(interaction_output) = interaction_outputs
                .iter()
                .find(|output| output.output == path)
                && !interaction_output
                    .choice_values
                    .iter()
                    .all(|value| is_boolean_interaction_choice_value(value))
            {
                issues.push(non_boolean_interaction_choice_condition_issue(
                    source,
                    process.key.process_id.as_ref(),
                    &gateway_id,
                    condition,
                    &path,
                    interaction_output,
                ));
                continue;
            }
            let Some(kind) = ambiguous_boolean_path_kind(&path) else {
                continue;
            };
            issues.push(ambiguous_boolean_condition_issue(
                source,
                process.key.process_id.as_ref(),
                &gateway_id,
                condition,
                &path,
                kind,
            ));
        }
    }
    issues
}

pub(super) fn ambiguous_boolean_gateway_condition_source_issues(
    source: &BpmnSourceFile,
) -> Vec<LintIssue> {
    let gateway_ids = collect_gateway_ids(&source.contents);
    if gateway_ids.is_empty() {
        return Vec::new();
    }

    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut current_process_id: Option<String> = None;
    let mut active_flow: Option<ActiveGatewayFlow> = None;
    let mut in_condition_expression = false;
    let mut condition_text = String::new();
    let mut issues = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "process") {
                    current_process_id = attribute_value(&reader, &event, "id");
                } else if is_element(&event, "sequenceFlow") {
                    if let Some(source_ref) = attribute_value(&reader, &event, "sourceRef")
                        && gateway_ids.contains(&source_ref)
                    {
                        active_flow = Some(ActiveGatewayFlow {
                            gateway_id: source_ref,
                            process_id: current_process_id
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            depth,
                        });
                    }
                } else if active_flow.is_some() && is_element(&event, "conditionExpression") {
                    in_condition_expression = true;
                    condition_text.clear();
                }
            }
            Ok(Event::Text(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok().as_deref().unwrap_or_default());
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok().as_deref().unwrap_or_default());
            }
            Ok(Event::GeneralRef(event)) if in_condition_expression => {
                let reference = event.decode().ok();
                append_entity_reference(&mut condition_text, reference.as_deref());
            }
            Ok(Event::End(event)) => {
                let event_name = event.name();
                let name = local_name(event_name.as_ref());
                if name == "conditionExpression" {
                    let condition = condition_text.trim();
                    if let Some(issue) = source_ambiguous_boolean_condition_issue(
                        source,
                        active_flow.as_ref(),
                        Some(condition),
                    ) {
                        issues.push(issue);
                    }
                    condition_text.clear();
                    in_condition_expression = false;
                }
                if active_flow.as_ref().is_some_and(|flow| flow.depth == depth) {
                    active_flow = None;
                }
                if name == "process" {
                    current_process_id = None;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return issues,
            Ok(_) => {}
        }
    }
}

pub(super) fn unsupported_gateway_condition_source_issues(
    source: &BpmnSourceFile,
) -> Vec<LintIssue> {
    let gateway_ids = collect_gateway_ids(&source.contents);
    if gateway_ids.is_empty() {
        return Vec::new();
    }

    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut current_process_id: Option<String> = None;
    let mut active_flow: Option<ActiveGatewayFlow> = None;
    let mut in_condition_expression = false;
    let mut condition_text = String::new();
    let mut conditions = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "process") {
                    current_process_id = attribute_value(&reader, &event, "id");
                } else if is_element(&event, "sequenceFlow") {
                    if let Some(source_ref) = attribute_value(&reader, &event, "sourceRef")
                        && gateway_ids.contains(&source_ref)
                    {
                        active_flow = Some(ActiveGatewayFlow {
                            gateway_id: source_ref,
                            process_id: current_process_id
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            depth,
                        });
                    }
                } else if active_flow.is_some() && is_element(&event, "conditionExpression") {
                    in_condition_expression = true;
                    condition_text.clear();
                }
            }
            Ok(Event::Text(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok().as_deref().unwrap_or_default());
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok().as_deref().unwrap_or_default());
            }
            Ok(Event::GeneralRef(event)) if in_condition_expression => {
                let reference = event.decode().ok();
                append_entity_reference(&mut condition_text, reference.as_deref());
            }
            Ok(Event::End(event)) => {
                let event_name = event.name();
                let name = local_name(event_name.as_ref());
                if name == "conditionExpression" {
                    let condition = condition_text.trim();
                    if let Some(condition) =
                        source_unsupported_gateway_condition(active_flow.as_ref(), Some(condition))
                    {
                        conditions.push(condition);
                    }
                    condition_text.clear();
                    in_condition_expression = false;
                }
                if active_flow.as_ref().is_some_and(|flow| flow.depth == depth) {
                    active_flow = None;
                }
                if name == "process" {
                    current_process_id = None;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => {
                return grouped_unsupported_gateway_condition_issues(source, conditions);
            }
            Ok(_) => {}
        }
    }
}

fn append_entity_reference(target: &mut String, reference: Option<&str>) {
    let Some(reference) = reference else {
        return;
    };
    if let Some(entity) = resolve_predefined_entity(reference) {
        target.push_str(entity);
    } else {
        target.push('&');
        target.push_str(reference);
        target.push(';');
    }
}

#[derive(Debug, Clone)]
struct ActiveGatewayFlow {
    gateway_id: String,
    process_id: String,
    depth: usize,
}

#[derive(Debug, Clone)]
struct UnsupportedGatewayCondition {
    process_id: String,
    gateway_id: String,
    condition: String,
}

fn source_ambiguous_boolean_condition_issue(
    source: &BpmnSourceFile,
    active_flow: Option<&ActiveGatewayFlow>,
    condition: Option<&str>,
) -> Option<LintIssue> {
    let flow = active_flow?;
    let condition = condition?.trim();
    let Some(GatewayConditionSummary::BooleanPath { path, .. }) =
        parse_gateway_condition_summary(condition)
    else {
        return None;
    };
    let kind = ambiguous_boolean_path_kind(&path)?;
    Some(ambiguous_boolean_condition_issue(
        source,
        &flow.process_id,
        &flow.gateway_id,
        condition,
        &path,
        kind,
    ))
}

fn source_unsupported_gateway_condition(
    active_flow: Option<&ActiveGatewayFlow>,
    condition: Option<&str>,
) -> Option<UnsupportedGatewayCondition> {
    let flow = active_flow?;
    let condition = condition?.trim();
    if condition.is_empty() || is_supported_gateway_condition(condition) {
        return None;
    }
    Some(UnsupportedGatewayCondition {
        process_id: flow.process_id.clone(),
        gateway_id: flow.gateway_id.clone(),
        condition: condition.to_string(),
    })
}

fn grouped_unsupported_gateway_condition_issues(
    source: &BpmnSourceFile,
    conditions: Vec<UnsupportedGatewayCondition>,
) -> Vec<LintIssue> {
    let mut groups: Vec<UnsupportedGatewayConditionGroup> = Vec::new();
    for condition in conditions {
        if let Some(group) = groups.iter_mut().find(|group| {
            group.process_id == condition.process_id && group.gateway_id == condition.gateway_id
        }) {
            group.conditions.push(condition.condition);
        } else {
            groups.push(UnsupportedGatewayConditionGroup {
                process_id: condition.process_id,
                gateway_id: condition.gateway_id,
                conditions: vec![condition.condition],
            });
        }
    }
    groups
        .into_iter()
        .map(|group| unsupported_gateway_condition_issue(source, group))
        .collect()
}

struct UnsupportedGatewayConditionGroup {
    process_id: String,
    gateway_id: String,
    conditions: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum AmbiguousBooleanPathKind {
    CountLike,
    ContentLike,
}

#[derive(Debug, Clone)]
struct StaticInteractionChoiceOutput {
    task_id: String,
    output: String,
    choice_values: Vec<String>,
}

#[derive(Default)]
struct ActiveInteractionChoiceOutput {
    task_id: String,
    result_output: Option<String>,
    choice_values: Vec<String>,
}

fn collect_static_interaction_choice_outputs(contents: &str) -> Vec<StaticInteractionChoiceOutput> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut current_user_task_id: Option<String> = None;
    let mut active_interaction: Option<ActiveInteractionChoiceOutput> = None;
    let mut outputs = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                if is_element(&event, "userTask") {
                    current_user_task_id = attribute_value(&reader, &event, "id");
                } else if is_element(&event, "interaction") {
                    if let Some(task_id) = current_user_task_id.as_ref() {
                        active_interaction = Some(ActiveInteractionChoiceOutput {
                            task_id: task_id.clone(),
                            ..ActiveInteractionChoiceOutput::default()
                        });
                    }
                } else if let Some(active) = active_interaction.as_mut() {
                    collect_interaction_choice_output_field(&reader, &event, active);
                }
            }
            Ok(Event::Empty(event)) => {
                if let Some(active) = active_interaction.as_mut() {
                    collect_interaction_choice_output_field(&reader, &event, active);
                }
            }
            Ok(Event::End(event)) => {
                let event_name = event.name();
                let name = local_name(event_name.as_ref());
                if name == "interaction"
                    && let Some(active) = active_interaction.take()
                    && let Some(output) = active.result_output
                    && !active.choice_values.is_empty()
                {
                    outputs.push(StaticInteractionChoiceOutput {
                        task_id: active.task_id,
                        output,
                        choice_values: active.choice_values,
                    });
                } else if name == "userTask" {
                    current_user_task_id = None;
                }
            }
            Ok(Event::Eof) | Err(_) => return outputs,
            Ok(_) => {}
        }
    }
}

fn collect_gateway_ids(contents: &str) -> HashSet<String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut ids = HashSet::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event))
                if is_element(&event, "exclusiveGateway")
                    || is_element(&event, "inclusiveGateway") =>
            {
                if let Some(id) = attribute_value(&reader, &event, "id") {
                    ids.insert(id);
                }
            }
            Ok(Event::Eof) | Err(_) => return ids,
            Ok(_) => {}
        }
    }
}

fn collect_interaction_choice_output_field(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    active: &mut ActiveInteractionChoiceOutput,
) {
    if is_element(event, "choice")
        && let Some(value) = attribute_value(reader, event, "value")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    {
        active.choice_values.push(value);
    } else if is_element(event, "result")
        && let Some(output) = attribute_value(reader, event, "output")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    {
        active.result_output = Some(output);
    }
}

fn is_boolean_interaction_choice_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true"
            | "false"
            | "yes"
            | "no"
            | "y"
            | "n"
            | "approved"
            | "approve"
            | "rejected"
            | "reject"
            | "accepted"
            | "accept"
            | "confirmed"
            | "confirm"
            | "continue"
            | "proceed"
            | "revise"
            | "revision"
            | "changes"
            | "declined"
            | "decline"
            | "denied"
            | "deny"
            | "stop"
            | "cancel"
            | "cancelled"
    )
}

fn is_count_like_boolean_path(path: &str) -> bool {
    let segment = path.rsplit('.').next().unwrap_or(path);
    let normalized = segment.to_ascii_lowercase();
    !is_boolean_shaped_name(&normalized)
        && [
            "count",
            "number",
            "total",
            "index",
            "length",
            "size",
            "amount",
            "remaining",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn is_content_like_boolean_path(path: &str) -> bool {
    let segment = path.rsplit('.').next().unwrap_or(path);
    let normalized = segment.to_ascii_lowercase();
    !is_boolean_shaped_name(&normalized)
        && !has_embedded_boolean_marker(segment, &normalized)
        && [
            "answer",
            "answers",
            "choice",
            "choices",
            "concern",
            "concerns",
            "detail",
            "details",
            "feedback",
            "guidance",
            "issue",
            "issues",
            "question",
            "questions",
            "response",
            "responses",
            "result",
            "results",
            "status",
        ]
        .iter()
        .any(|marker| normalized.ends_with(marker))
}

fn has_embedded_boolean_marker(segment: &str, normalized: &str) -> bool {
    ["Is", "Has", "Can", "Should", "Needs", "Need", "Did", "Will"]
        .iter()
        .any(|marker| segment.contains(marker))
        || [
            "_is_", "_has_", "_can_", "_should_", "_needs_", "_need_", "_did_", "_will_", "-is-",
            "-has-", "-can-", "-should-", "-needs-", "-need-", "-did-", "-will-",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn ambiguous_boolean_path_kind(path: &str) -> Option<AmbiguousBooleanPathKind> {
    if is_count_like_boolean_path(path) {
        return Some(AmbiguousBooleanPathKind::CountLike);
    }
    if is_content_like_boolean_path(path) {
        return Some(AmbiguousBooleanPathKind::ContentLike);
    }
    None
}

fn is_boolean_shaped_name(normalized: &str) -> bool {
    ["is", "has", "can", "should", "needs", "need", "did", "will"]
        .iter()
        .any(|prefix| normalized.starts_with(prefix))
}

fn non_boolean_interaction_choice_condition_issue(
    source: &BpmnSourceFile,
    process_id: &str,
    gateway_id: &str,
    condition: &str,
    variable_path: &str,
    interaction_output: &StaticInteractionChoiceOutput,
) -> LintIssue {
    let choice_values = interaction_output.choice_values.join(", ");
    let issue = LintIssue::new(
        "bpmn.non_boolean_interaction_choice_condition",
        "Gateway boolean condition consumes non-boolean interaction choices",
        format!(
            "Process '{process_id}' gateway '{gateway_id}' uses bare boolean condition '{condition}', but userTask '{}' maps static choice values [{choice_values}] into '{variable_path}'.",
            interaction_output.task_id
        ),
        "The bounded runtime evaluates bare gateway condition paths as JSON booleans only. A user interaction result that drives such a gateway must produce a boolean-shaped answer, or an upstream service task must derive a separate boolean output.",
        vec![
            format!(
                "For a boolean branch, make userTask '{}' return boolean/approval values such as `true`/`false`, `yes`/`no`, or `approved`/`rejected` into '{variable_path}'.",
                interaction_output.task_id
            ),
            format!(
                "Prefer a boolean-shaped output name such as `needsMoreQuestions` or `hasMoreQuestions`, then route with `<conditionExpression>{variable_path}</conditionExpression>` only when the host output is a JSON boolean."
            ),
            "If the choices are semantic strings, keep that result as a text answer and add a serviceTask that emits a separate JSON boolean consumed by the gateway.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' by aligning userTask '{}' choice output '{variable_path}' with gateway '{gateway_id}': either use boolean/approval choice values for the gateway variable, or add a serviceTask that converts semantic choice strings into a JSON boolean before routing.",
            interaction_output.task_id
        ),
        json!({
            "process_id": process_id,
            "gateway_id": gateway_id,
            "condition": condition,
            "user_task_id": interaction_output.task_id,
            "output": variable_path,
            "choice_values": interaction_output.choice_values,
            "expected_runtime_type": "json_boolean for bare gateway condition path"
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.gateway.condition.v1",
        "strategy": "align_interaction_choice_output_with_boolean_gateway",
        "target": {
            "process_id": process_id,
            "gateway_id": gateway_id,
            "user_task_id": interaction_output.task_id,
            "condition": condition,
            "output": variable_path,
            "choice_values": interaction_output.choice_values
        },
        "actions": [{
            "op": "choose_one",
            "examples": [
                "<qianji:choice value=\"true\" label=\"Ask another question\"/>",
                "<qianji:choice value=\"false\" label=\"Proceed\"/>",
                "<qianji:result output=\"needsMoreQuestions\"/>",
                "<conditionExpression xsi:type=\"tFormalExpression\">needsMoreQuestions</conditionExpression>"
            ],
            "options": [
                {
                    "op": "use_boolean_choice_values",
                    "examples": [
                        "<qianji:choice value=\"true\" label=\"Ask another question\"/>",
                        "<qianji:choice value=\"false\" label=\"Proceed\"/>",
                        "<qianji:result output=\"needsMoreQuestions\"/>",
                        "<conditionExpression xsi:type=\"tFormalExpression\">needsMoreQuestions</conditionExpression>"
                    ],
                    "requires": "the host maps the interaction result to a JSON boolean before completing the userTask"
                },
                {
                    "op": "derive_boolean_with_service_task",
                    "when": "choice values must remain semantic strings",
                    "requires": "serviceTask outputs a separate JSON boolean such as needsMoreQuestions, and the gateway routes on that boolean"
                }
            ],
            "forbidden_forms": [
                "semantic choice strings such as need_more_clarification routed directly by a bare boolean condition",
                "bare gateway condition over a userTask text answer",
                "string values pretending to be booleans without a host/runtime mapping"
            ]
        }]
    }));

    let Some(span) =
        find_condition_expression_span(source.contents.as_str(), gateway_id, condition)
    else {
        return issue;
    };
    issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        "this bare gateway condition needs a JSON boolean",
        format!(
            "The condition consumes userTask '{}' choices [{choice_values}] through '{variable_path}'. Convert the interaction result to a boolean output, or derive a separate boolean before this gateway.",
            interaction_output.task_id
        ),
    ))
}

fn ambiguous_boolean_condition_issue(
    source: &BpmnSourceFile,
    process_id: &str,
    gateway_id: &str,
    condition: &str,
    variable_path: &str,
    kind: AmbiguousBooleanPathKind,
) -> LintIssue {
    let (title, summary, why_it_failed, repair_guidance, llm_fix_prompt, valid_repairs) =
        ambiguous_boolean_condition_guidance(
            process_id,
            gateway_id,
            condition,
            variable_path,
            kind,
        );
    let issue = LintIssue::new(
        "bpmn.ambiguous_boolean_gateway_condition",
        title,
        summary,
        why_it_failed,
        repair_guidance,
        llm_fix_prompt,
        json!({
            "process_id": process_id,
            "gateway_id": gateway_id,
            "condition": condition,
            "variable_path": variable_path,
            "expected_runtime_type": "boolean for bare path, number for numeric comparison",
            "valid_repairs": valid_repairs
        }),
    )
    .with_structured_repair(ambiguous_boolean_condition_repair(
        process_id,
        gateway_id,
        condition,
        variable_path,
        kind,
    ));

    let Some(span) = find_condition_expression_span(&source.contents, gateway_id, condition) else {
        return issue;
    };
    issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        "bare condition paths must resolve to JSON boolean values",
        ambiguous_boolean_condition_help(variable_path, kind),
    ))
}

fn unsupported_gateway_condition_issue(
    source: &BpmnSourceFile,
    group: UnsupportedGatewayConditionGroup,
) -> LintIssue {
    let process_id = group.process_id;
    let gateway_id = group.gateway_id;
    let conditions = group.conditions;
    let first_condition = conditions.first().cloned().unwrap_or_default();
    let condition_list = conditions
        .iter()
        .map(|condition| format!("`{condition}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let issue = LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Conditional gateway condition exceeds the bounded subset",
        format!(
            "Process '{process_id}' gateway '{gateway_id}' uses unsupported conditionExpression value(s): {condition_list}."
        ),
        "The bounded runtime accepts only boolean variable paths such as `approved` or `not approved`, dotted boolean paths such as `flags.approved`, and numeric comparisons against numeric literals such as `amount > 100`. String or enum equality must be converted into an upstream boolean route output.",
        vec![
            "Rewrite the branch condition as one boolean variable path, optionally prefixed with `not`, or as one numeric comparison from an identifier path to one numeric literal.".to_string(),
            "For string or enum decisions, add one top-level boolean qianji output on the upstream task, then route on that boolean output.".to_string(),
            "If the string or enum value comes from a userTask `<qianji:result output=\"...\"/>`, keep that result output declared on the userTask and add a following serviceTask to derive route booleans. Do not replace the userTask's qianji:outputs with derived booleans.".to_string(),
            "Do not use string equality, boolean-literal comparisons, variable-to-variable comparisons, function calls, scripts, or logical combinations in gateway conditions.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' gateway '{gateway_id}' by replacing every unsupported conditionExpression in [{condition_list}] with bounded boolean route variables or numeric comparisons. If these conditions compare user choice/status strings, preserve the original qianji:result output on the userTask, add a following serviceTask that consumes it and emits JSON boolean route outputs, then route on those booleans."
        ),
        json!({
            "process_id": process_id,
            "gateway_id": gateway_id,
            "conditions": conditions,
            "valid_condition_forms": [
                "approved",
                "not approved",
                "flags.approved",
                "amount > 100",
                "risk >= 7"
            ],
            "forbidden_condition_forms": [
                "taskStatus == \"completed\"",
                "choice == 'merge'",
                "approved == true",
                "currentCount > requiredCount",
                "approved and verified"
            ]
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
            "contract": "qianji.bpmn.gateway.condition.v1",
            "strategy": "rewrite_condition_to_bounded_subset",
            "target": {
                "process_id": process_id,
                "gateway_id": gateway_id,
                "conditions": conditions
            },
            "actions": [{
            "op": "replace_unsupported_conditions",
            "allowed_forms": [
                "boolean variable path",
                "not boolean variable path",
                "numeric variable path compared to numeric literal"
            ],
            "examples": ["taskCompleted", "not blocked", "questionsRemaining > 0"],
            "producer_change": "if the existing condition compares a string/enum/status from a userTask qianji:result, keep that result output declared on the userTask, then add a following serviceTask that consumes it and declares/emits JSON boolean route outputs",
            "forbid": "replacing a userTask qianji:outputs list with derived booleans while qianji:result still points at the original answer",
            "forbidden_forms": [
                "taskStatus == \"completed\"",
                "choice == 'merge'",
                "approved == true",
                "approved and verified",
                "currentCount > requiredCount"
            ]
        }]
    }));

    let Some(span) =
        find_condition_expression_span(&source.contents, &gateway_id, &first_condition)
    else {
        return issue;
    };
    issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        "rewrite this condition into qianji's bounded subset",
        "Use a JSON boolean route variable such as `taskCompleted`, or a numeric comparison against a literal. For status or choice strings, emit a separate boolean output upstream and route on it.",
    ))
}

fn ambiguous_boolean_condition_guidance(
    process_id: &str,
    gateway_id: &str,
    condition: &str,
    variable_path: &str,
    kind: AmbiguousBooleanPathKind,
) -> (
    String,
    String,
    String,
    Vec<String>,
    String,
    serde_json::Value,
) {
    match kind {
        AmbiguousBooleanPathKind::CountLike => (
            "Gateway boolean condition uses a count-like variable".to_string(),
            format!(
                "Process '{process_id}' gateway '{gateway_id}' uses boolean condition '{condition}', but variable '{variable_path}' is count-like and commonly resolves to a JSON number."
            ),
            "The bounded runtime evaluates bare gateway condition paths as JSON booleans only. Numeric counters must use an explicit numeric comparison, or the producer must emit a boolean-shaped variable.".to_string(),
            vec![
                format!(
                    "If '{variable_path}' is a count, rewrite the branch condition to `{variable_path} > 0` and ensure the upstream task emits a JSON number."
                ),
                "If the branch is boolean, rename the output to a boolean-shaped variable such as `hasMoreQuestions` or `needsMoreQuestions` and emit only `true` or `false`.".to_string(),
                "Keep the fallback branch as the gateway `default` flow without a conditionExpression.".to_string(),
            ],
            format!(
                "Repair process '{process_id}' gateway '{gateway_id}' by replacing boolean-path condition `{condition}` with either `{variable_path} > 0` for numeric counts, or by renaming the upstream qianji output to a boolean-shaped variable and routing on that boolean."
            ),
            json!([
                {
                    "conditionExpression": format!("{variable_path} > 0"),
                    "producer_output_type": "json_number"
                },
                {
                    "conditionExpression": "hasMoreQuestions",
                    "producer_output_type": "json_boolean"
                }
            ]),
        ),
        AmbiguousBooleanPathKind::ContentLike => (
            "Gateway boolean condition uses a content-like variable".to_string(),
            format!(
                "Process '{process_id}' gateway '{gateway_id}' uses boolean condition '{condition}', but variable '{variable_path}' is content-like and commonly resolves to text, arrays, objects, or enum strings."
            ),
            "The bounded runtime evaluates bare gateway condition paths as JSON booleans only. User-facing content, status strings, arrays, and details must stay separate from route booleans.".to_string(),
            vec![
                format!(
                    "Keep '{variable_path}' as content for prompts or task inputs, but do not route directly on it."
                ),
                "Add a separate boolean-shaped route output such as `hasQuestions`, `hasConcerns`, `needsHumanInput`, or `shouldEscalate`, and emit only JSON true or false.".to_string(),
                "Route the gateway on that boolean output, and make every qianji task that can enter this gateway declare and produce the same route boolean.".to_string(),
            ],
            format!(
                "Repair process '{process_id}' gateway '{gateway_id}' by replacing content-like boolean condition `{condition}` with a separate boolean route variable such as `hasQuestions`, `hasConcerns`, `needsHumanInput`, or `shouldEscalate`. Preserve `{variable_path}` for user-facing text or arrays, add that route boolean to the upstream qianji outputs/prompts, and route only on the JSON boolean."
            ),
            json!([
                {
                    "conditionExpression": "hasQuestions",
                    "producer_output_type": "json_boolean",
                    "content_variable": variable_path
                },
                {
                    "conditionExpression": "hasConcerns",
                    "producer_output_type": "json_boolean",
                    "content_variable": variable_path
                },
                {
                    "conditionExpression": "shouldEscalate",
                    "producer_output_type": "json_boolean",
                    "content_variable": variable_path
                }
            ]),
        ),
    }
}

fn ambiguous_boolean_condition_help(variable_path: &str, kind: AmbiguousBooleanPathKind) -> String {
    match kind {
        AmbiguousBooleanPathKind::CountLike => format!(
            "Use `{variable_path} > 0` for a numeric count, or rename the producer output to a boolean-shaped variable and emit true/false."
        ),
        AmbiguousBooleanPathKind::ContentLike => format!(
            "Keep `{variable_path}` as content, add a separate boolean-shaped route output, and route on that JSON boolean."
        ),
    }
}

fn ambiguous_boolean_condition_repair(
    process_id: &str,
    gateway_id: &str,
    condition: &str,
    variable_path: &str,
    kind: AmbiguousBooleanPathKind,
) -> serde_json::Value {
    match kind {
        AmbiguousBooleanPathKind::CountLike => json!({
            "schema_version": 1,
            "contract": "qianji.bpmn.gateway.condition.v1",
            "strategy": "disambiguate_count_like_boolean_condition",
            "target": {
                "process_id": process_id,
                "gateway_id": gateway_id,
                "condition": condition,
                "variable_path": variable_path
            },
            "actions": [
                {
                    "op": "choose_one",
                    "examples": [
                        format!("<conditionExpression xsi:type=\"tFormalExpression\">{variable_path} > 0</conditionExpression>"),
                        format!("Return JSON with currentQuestion and {variable_path} as a JSON number."),
                        "<qianji:outputs>currentQuestion,hasMoreQuestions</qianji:outputs> with <conditionExpression>hasMoreQuestions</conditionExpression>".to_string()
                    ],
                    "forbidden_forms": [
                        format!("<conditionExpression>{condition}</conditionExpression>"),
                        format!("{variable_path} as a string such as \"5\""),
                        format!("{variable_path} as a boolean while the gateway uses `{variable_path} > 0`"),
                        "routing count-like variables through bare boolean paths".to_string()
                    ],
                    "options": [
                        {
                            "op": "rewrite_condition_expression",
                            "from": condition,
                            "to": format!("{variable_path} > 0"),
                            "requires": "upstream qianji task emits a JSON number",
                            "producer_change": format!("update every qianji task that outputs `{variable_path}` so its prompt says `{variable_path}` is a JSON number count, not a boolean or string")
                        },
                        {
                            "op": "rename_gateway_variable_to_boolean",
                            "examples": ["hasMoreQuestions", "needsMoreQuestions"],
                            "requires": "upstream qianji task emits true or false",
                            "producer_change": format!("rename every qianji output/input/condition use of `{variable_path}` consistently if choosing the boolean route")
                        }
                    ]
                }
            ]
        }),
        AmbiguousBooleanPathKind::ContentLike => json!({
            "schema_version": 1,
            "contract": "qianji.bpmn.gateway.condition.v1",
            "strategy": "split_content_variable_from_boolean_route",
            "target": {
                "process_id": process_id,
                "gateway_id": gateway_id,
                "condition": condition,
                "content_variable": variable_path
            },
            "actions": [
                {
                    "op": "add_boolean_route_output",
                    "examples": ["hasQuestions", "hasConcerns", "needsHumanInput", "shouldEscalate"],
                    "producer_change": format!("update every qianji task that can reach gateway `{gateway_id}` so it declares and emits a JSON boolean route variable separate from `{variable_path}`"),
                    "route_change": format!("replace `<conditionExpression>{condition}</conditionExpression>` with the new boolean route variable"),
                    "preserve": format!("keep `{variable_path}` available for user-facing question text, concerns, details, or other content")
                }
            ],
            "forbidden_forms": [
                format!("<conditionExpression>{condition}</conditionExpression>"),
                "routing directly on text, arrays, objects, or enum status values".to_string(),
                "using the same variable as both user-facing prompt content and boolean route signal".to_string()
            ]
        }),
    }
}

fn find_condition_expression_span(
    contents: &str,
    gateway_id: &str,
    condition: &str,
) -> Option<std::ops::Range<usize>> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut sequence_flow_depth = None;
    let mut depth = 0usize;
    let mut in_condition_expression = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "sequenceFlow")
                    && attribute_value(&reader, &event, "sourceRef").as_deref() == Some(gateway_id)
                {
                    sequence_flow_depth = Some(depth);
                } else if sequence_flow_depth.is_some() && is_element(&event, "conditionExpression")
                {
                    in_condition_expression = true;
                }
            }
            Ok(Event::Empty(event)) => {
                if is_element(&event, "sequenceFlow")
                    && attribute_value(&reader, &event, "sourceRef").as_deref() == Some(gateway_id)
                {
                    sequence_flow_depth = None;
                }
            }
            Ok(Event::Text(event)) if in_condition_expression => {
                let text = event.decode().ok()?;
                if text.trim() == condition {
                    return event_text_span(reader_position(&reader)?, event.as_ref());
                }
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                let text = event.decode().ok()?;
                if text.trim() == condition {
                    return event_text_span(reader_position(&reader)?, event.as_ref());
                }
            }
            Ok(Event::End(event)) => {
                if local_name(event.name().as_ref()) == "conditionExpression" {
                    in_condition_expression = false;
                }
                if sequence_flow_depth == Some(depth) {
                    sequence_flow_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn is_element(event: &BytesStart<'_>, local: &str) -> bool {
    local_name(event.name().as_ref()) == local
}

fn local_name(raw: &[u8]) -> &str {
    std::str::from_utf8(raw)
        .ok()
        .map_or("", |name| name.rsplit(':').next().unwrap_or(name))
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

fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
}

fn event_text_span(event_end: usize, raw_text: &[u8]) -> Option<std::ops::Range<usize>> {
    let end = event_end;
    let start = end.checked_sub(raw_text.len())?;
    Some(start..end)
}
