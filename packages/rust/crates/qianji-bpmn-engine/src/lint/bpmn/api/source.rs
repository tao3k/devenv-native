use super::{
    MissingBranchDuplicateKind, duplicate_unconditional_repair_guidance,
    missing_branch_duplicate_kind, missing_condition_help, missing_condition_label,
    missing_condition_llm_prompt, missing_condition_structured_repair,
};
mod xml;

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::BpmnEngineError;
use crate::lint_api::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use serde_json::json;
pub(super) use xml::{
    escaped_line_fix_for_ampersand, find_gateway_condition_expression_span,
    find_gateway_condition_expression_text, find_missing_branch_condition_context,
    find_unescaped_ampersand_span, find_unescaped_placeholder_span, find_xml_error_token_span,
    malformed_closing_tag_line_fix, unsupported_condition_expression_help,
};
use xml::{
    find_bounded_gateway_ids, find_gateway_default_span_and_id, find_outgoing_flow_summaries,
    find_routable_task_spans,
};

pub(super) struct InvalidDefaultFlowContext {
    pub(super) default_flow_id: String,
    pub(super) gateway_span: std::ops::Range<usize>,
    pub(super) outgoing_flows: Vec<OutgoingFlowSummary>,
}

pub(super) struct DefaultBranchingContext {
    pub(super) default_flow_id: String,
    pub(super) gateway_span: std::ops::Range<usize>,
    pub(super) outgoing_flows: Vec<OutgoingFlowSummary>,
}

pub(super) struct TaskRoutingViolation {
    pub(super) task_id: String,
    pub(super) task_span: std::ops::Range<usize>,
    pub(super) outgoing_flows: Vec<OutgoingFlowSummary>,
}

#[derive(Clone)]
pub(super) struct OutgoingFlowSummary {
    pub(super) id: String,
    pub(super) has_condition: bool,
}

pub(super) struct MissingBranchConditionContext {
    pub(super) flow_id: String,
    pub(super) target_ref: Option<String>,
    pub(super) flow_span: std::ops::Range<usize>,
    pub(super) duplicate_conditioned_flow_ids: Vec<String>,
    pub(super) duplicate_default_flow_ids: Vec<String>,
}

struct GatewayFlowDetail {
    id: String,
    target_ref: Option<String>,
    span: std::ops::Range<usize>,
    has_condition: bool,
    is_default: bool,
}

struct ActiveGatewayFlow {
    depth: usize,
    id: String,
    target_ref: Option<String>,
    span: std::ops::Range<usize>,
    has_condition: bool,
    is_default: bool,
}

pub(super) fn preferred_default_flow(
    context: &InvalidDefaultFlowContext,
) -> Option<&OutgoingFlowSummary> {
    context
        .outgoing_flows
        .iter()
        .find(|flow| looks_like_renamed_default(&context.default_flow_id, &flow.id))
        .or_else(|| {
            context
                .outgoing_flows
                .iter()
                .find(|flow| !flow.has_condition)
        })
        .or_else(|| context.outgoing_flows.first())
}

fn looks_like_renamed_default(stale_default_id: &str, candidate_id: &str) -> bool {
    if stale_default_id.is_empty() || candidate_id.is_empty() {
        return false;
    }
    let stale = stale_default_id.to_ascii_lowercase();
    let candidate = candidate_id.to_ascii_lowercase();
    candidate.starts_with(&stale) || stale.starts_with(&candidate)
}

pub(super) fn find_invalid_default_flow_context(
    contents: &str,
    gateway_id: &str,
) -> Option<InvalidDefaultFlowContext> {
    let (gateway_span, default_flow_id) = find_gateway_default_span_and_id(contents, gateway_id)?;
    let outgoing_flows = find_outgoing_flow_summaries(contents, gateway_id);
    if outgoing_flows.is_empty() {
        return None;
    }
    Some(InvalidDefaultFlowContext {
        default_flow_id,
        gateway_span,
        outgoing_flows,
    })
}

pub(super) fn find_default_branching_context(
    contents: &str,
    gateway_id: &str,
) -> Option<DefaultBranchingContext> {
    let (gateway_span, default_flow_id) = find_gateway_default_span_and_id(contents, gateway_id)?;
    Some(DefaultBranchingContext {
        default_flow_id,
        gateway_span,
        outgoing_flows: find_outgoing_flow_summaries(contents, gateway_id),
    })
}

pub(super) fn should_append_source_task_routing_issue(error: &BpmnEngineError) -> bool {
    matches!(error, BpmnEngineError::UnknownSequenceFlowEndpoint { .. })
}

pub(super) fn should_append_source_gateway_condition_issues(error: &BpmnEngineError) -> bool {
    !matches!(
        error,
        BpmnEngineError::InvalidXml { .. }
            | BpmnEngineError::UnsupportedGatewayConfiguration {
                detail: "missing_condition_expression",
                ..
            }
            | BpmnEngineError::UnsupportedGatewayConfiguration {
                detail: "unknown_default_flow" | "default_flow_not_outgoing",
                ..
            }
            | BpmnEngineError::UnsupportedGatewayConfiguration {
                detail: "default_flow_requires_multiple_outgoing",
                ..
            }
    )
}

pub(super) fn should_append_source_unsupported_condition_issues(error: &BpmnEngineError) -> bool {
    !matches!(error, BpmnEngineError::InvalidXml { .. })
}

pub(super) fn append_unique_source_issues(issues: &mut Vec<LintIssue>, candidates: Vec<LintIssue>) {
    for candidate in candidates {
        if let Some(position) = issues
            .iter()
            .position(|existing| same_source_issue(existing, &candidate))
        {
            if source_issue_group_size(&candidate) > source_issue_group_size(&issues[position]) {
                issues[position] = candidate;
            }
        } else {
            issues.push(candidate);
        }
    }
}

fn same_source_issue(left: &LintIssue, right: &LintIssue) -> bool {
    if left.code != right.code {
        return false;
    }
    match (&left.source_diagnostic, &right.source_diagnostic) {
        (Some(left), Some(right)) => left.source_id == right.source_id && left.span == right.span,
        _ => left.summary == right.summary,
    }
}

pub(super) fn source_issue_group_size(issue: &LintIssue) -> usize {
    issue
        .evidence
        .get("conditions")
        .and_then(|value| value.as_array())
        .map_or(1, Vec::len)
}

pub(super) fn source_duplicate_unconditional_gateway_issues(
    source: &BpmnSourceFile,
) -> Vec<LintIssue> {
    find_bounded_gateway_ids(&source.contents)
        .into_iter()
        .filter_map(|gateway_id| {
            let context = find_missing_branch_condition_context(&source.contents, &gateway_id)?;
            (missing_branch_duplicate_kind(
                &context.duplicate_conditioned_flow_ids,
                &context.duplicate_default_flow_ids,
            ) != MissingBranchDuplicateKind::None)
                .then(|| duplicate_unconditional_gateway_issue(source, &gateway_id, &context))
        })
        .collect()
}

pub(super) fn source_invalid_default_gateway_issues(source: &BpmnSourceFile) -> Vec<LintIssue> {
    find_bounded_gateway_ids(&source.contents)
        .into_iter()
        .filter_map(|gateway_id| {
            let context = find_invalid_default_flow_context(&source.contents, &gateway_id)?;
            let default_is_outgoing = context
                .outgoing_flows
                .iter()
                .any(|flow| flow.id == context.default_flow_id);
            (!default_is_outgoing)
                .then(|| invalid_default_gateway_issue(source, &gateway_id, &context))
        })
        .collect()
}

fn invalid_default_gateway_issue(
    source: &BpmnSourceFile,
    gateway_id: &str,
    context: &InvalidDefaultFlowContext,
) -> LintIssue {
    let invalid_default_flow_id = context.default_flow_id.clone();
    let valid_flow_ids = context
        .outgoing_flows
        .iter()
        .map(|flow| flow.id.clone())
        .collect::<Vec<_>>();
    let valid_list = valid_flow_ids.join(", ");
    let preferred_flow = preferred_default_flow(context);
    let preferred_flow_id = preferred_flow.map(|flow| flow.id.clone());
    let preferred = preferred_flow_id
        .as_deref()
        .unwrap_or("one existing outgoing sequenceFlow id from this gateway");
    let preferred_has_condition = preferred_flow.is_some_and(|flow| flow.has_condition);
    let condition_help = if preferred_has_condition {
        " Remove that sequenceFlow conditionExpression when making it the default branch."
    } else {
        ""
    };
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Conditional gateway default flow reference is invalid",
        format!(
            "Gateway '{gateway_id}' marks default flow '{invalid_default_flow_id}', but that id is not one of this gateway's outgoing flows."
        ),
        "A bounded conditional gateway default must reference one sequenceFlow whose sourceRef is the same gateway. Missing or stale default ids fail before runtime routing is safe.",
        vec![
            format!(
                "Replace `default=\"{invalid_default_flow_id}\"` on gateway '{gateway_id}' with one valid outgoing flow id from this same gateway: {valid_list}."
            ),
            format!(
                "Prefer `default=\"{preferred}\"` when it matches the intended fallback."
            ),
            "Keep the selected default branch unconditional; conditions belong only on non-default outgoing branches.".to_string(),
        ],
        format!(
            "Repair gateway '{gateway_id}' by changing stale default '{invalid_default_flow_id}' to one valid outgoing sequenceFlow id: {valid_list}. Prefer `{preferred}` if it is the fallback branch."
        ),
        json!({
            "gateway_id": gateway_id,
            "invalid_default_flow_id": invalid_default_flow_id.clone(),
            "valid_outgoing_flow_ids": valid_flow_ids,
            "preferred_default_flow_id": preferred_flow_id,
            "preferred_default_has_condition": preferred_has_condition,
        }),
    )
    .with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(context.gateway_span.start, context.gateway_span.end),
        format!("retarget stale default flow `{invalid_default_flow_id}`"),
        format!(
            "Valid outgoing flow ids from gateway `{gateway_id}`: {valid_list}. Prefer `default=\"{preferred}\"` when it is the intended fallback.{condition_help}"
        ),
    ))
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "bpmn.native.gateway.bounded.v1",
        "strategy": "retarget_default_flow_to_existing_outgoing",
        "actions": [{
            "op": "set_gateway_default",
            "target": format!("gateway#{gateway_id}"),
            "current": invalid_default_flow_id,
            "valid_outgoing_flow_ids": valid_flow_ids,
            "preferred_default_flow_id": preferred_flow_id,
            "preferred_default_has_condition": preferred_has_condition,
            "also": "remove conditionExpression from the selected default sequenceFlow if it has one",
            "examples": context
                .outgoing_flows
                .iter()
                .map(|flow| format!("default=\"{}\"", flow.id))
                .collect::<Vec<_>>(),
            "forbidden_forms": [
                "default references a missing sequenceFlow id",
                "default references a sequenceFlow whose sourceRef is another node"
            ],
            "forbid": "missing flow ids or flow ids owned by another source"
        }]
    }))
}

fn duplicate_unconditional_gateway_issue(
    source: &BpmnSourceFile,
    gateway_id: &str,
    context: &MissingBranchConditionContext,
) -> LintIssue {
    let flow_id = context.flow_id.clone();
    let duplicate_conditioned_flow_ids = context.duplicate_conditioned_flow_ids.clone();
    let duplicate_default_flow_ids = context.duplicate_default_flow_ids.clone();
    let duplicate_kind =
        missing_branch_duplicate_kind(&duplicate_conditioned_flow_ids, &duplicate_default_flow_ids);
    let duplicate_ids = match duplicate_kind {
        MissingBranchDuplicateKind::Conditioned => duplicate_conditioned_flow_ids.join(", "),
        MissingBranchDuplicateKind::Default => duplicate_default_flow_ids.join(", "),
        MissingBranchDuplicateKind::None => String::new(),
    };
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Non-default bounded conditional branches need conditions",
        format!(
            "Gateway '{gateway_id}' has unconditional non-default sequenceFlow '{flow_id}' duplicating existing branch(es): {duplicate_ids}."
        ),
        "Every non-default outgoing branch of a bounded conditional gateway needs a condition. If an unconditional branch duplicates an existing branch to the same target, remove the duplicate instead of adding a second route.",
        duplicate_unconditional_repair_guidance(&flow_id, &duplicate_ids, duplicate_kind),
        missing_condition_llm_prompt(gateway_id, &flow_id, &duplicate_ids, duplicate_kind),
        json!({
            "gateway_id": gateway_id,
            "duplicate_unconditional_flow_id": flow_id.clone(),
            "duplicate_conditioned_flow_ids": duplicate_conditioned_flow_ids.clone(),
            "duplicate_default_flow_ids": duplicate_default_flow_ids.clone(),
        }),
    )
    .with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(context.flow_span.start, context.flow_span.end),
        missing_condition_label(duplicate_kind),
        missing_condition_help(duplicate_kind, &duplicate_ids, &flow_id),
    ))
    .with_structured_repair(missing_condition_structured_repair(
        gateway_id,
        &flow_id,
        context.target_ref.as_ref(),
        &duplicate_conditioned_flow_ids,
        &duplicate_default_flow_ids,
        duplicate_kind,
    ))
}

pub(super) fn source_task_routing_issue(source: &BpmnSourceFile) -> Option<LintIssue> {
    let violations = find_task_routing_violations(&source.contents);
    let primary = violations.first()?;
    let all_violations_summary = task_routing_violations_summary(&violations);
    let issue = LintIssue::new(
        "bpmn.unsupported_task_configuration",
        "Executable tasks must have exactly one outgoing sequence flow",
        format!(
            "Found {} executable task route violation(s): {all_violations_summary}.",
            violations.len(),
        ),
        "Every executable task in the bounded runtime routes by completing the task and taking exactly one outgoing `sequenceFlow`. Branching belongs behind a gateway, not directly on the task, and a task with no outgoing route will fail at runtime after host completion.",
        vec![
            "Repair every listed task in the same patch; do not fix only the first source span.".to_string(),
            "If a task should continue, add one outgoing `sequenceFlow` from that task to the next BPMN node.".to_string(),
            "If a task result should branch, route the task to one `exclusiveGateway` and put conditional/default branches on that gateway.".to_string(),
        ],
        format!(
            "Repair all executable task route violations in '{}': {all_violations_summary}. Preserve task ids and host configs.",
            source.source_id,
        ),
        json!({
            "all_task_route_violations": task_routing_violations_json(&violations),
        }),
    )
    .with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(primary.task_span.start, primary.task_span.end),
        "task must have exactly one outgoing sequenceFlow",
        format!(
            "This is the first task route violation. Repair all listed task route violations in the same patch: {all_violations_summary}."
        ),
    ))
    .with_structured_repair(task_routing_structured_repair(&violations));
    Some(issue)
}

pub(super) fn find_task_routing_violations(contents: &str) -> Vec<TaskRoutingViolation> {
    find_routable_task_spans(contents)
        .into_iter()
        .filter_map(|(task_id, task_span)| {
            let outgoing_flows = find_outgoing_flow_summaries(contents, &task_id);
            (outgoing_flows.len() != 1).then_some(TaskRoutingViolation {
                task_id,
                task_span,
                outgoing_flows,
            })
        })
        .collect()
}

pub(super) fn task_routing_violations_summary(violations: &[TaskRoutingViolation]) -> String {
    violations
        .iter()
        .map(|violation| {
            let outgoing = violation
                .outgoing_flows
                .iter()
                .map(|flow| flow.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let outgoing = if outgoing.is_empty() {
                "none".to_string()
            } else {
                outgoing
            };
            format!(
                "{}({} outgoing: {outgoing})",
                violation.task_id,
                violation.outgoing_flows.len(),
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

pub(super) fn task_routing_violations_json(
    violations: &[TaskRoutingViolation],
) -> serde_json::Value {
    json!(
        violations
            .iter()
            .map(|violation| {
                json!({
                    "task_id": violation.task_id.clone(),
                    "outgoing_flow_ids": violation
                        .outgoing_flows
                        .iter()
                        .map(|flow| flow.id.clone())
                        .collect::<Vec<_>>(),
                    "outgoing_flow_count": violation.outgoing_flows.len(),
                })
            })
            .collect::<Vec<_>>()
    )
}

pub(super) fn task_routing_structured_repair(
    violations: &[TaskRoutingViolation],
) -> serde_json::Value {
    json!({
        "schema_version": 1,
        "contract": "bpmn.native.task.routing.v1",
        "strategy": "repair_task_single_outgoing_route",
        "target": {
            "task_route_violations": task_routing_violations_json(violations)
        },
        "actions": [{
            "op": "repair_all_task_routes",
            "target": task_routing_violations_summary(violations),
            "requires": "repair every listed task route violation in the same patch",
            "examples": [
                "<sequenceFlow id=\"Flow_Task_Next\" sourceRef=\"Task_Id\" targetRef=\"Next_Node\"/>",
                "Task_Id -> ExclusiveGateway_Id when branching is required"
            ],
            "options": [
                {
                    "op": "add_sequence_flow",
                    "when": "the task should continue to one known next node"
                },
                {
                    "op": "route_task_to_gateway",
                    "when": "the task result should branch after completion"
                },
                {
                    "op": "collapse_multiple_task_flows_behind_gateway",
                    "when": "the task currently has multiple outgoing routes"
                }
            ],
            "forbidden_forms": [
                "task with zero outgoing sequenceFlow",
                "task with multiple outgoing sequenceFlows",
                "conditional branches directly attached to a task"
            ]
        }]
    })
}
