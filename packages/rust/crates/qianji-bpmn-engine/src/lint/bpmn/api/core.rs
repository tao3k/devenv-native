//! Coordinates BPMN lint report assembly across parser output, source diagnostics,
//! and rule-specific issue producers.

use super::source::{
    InvalidDefaultFlowContext, append_unique_source_issues, escaped_line_fix_for_ampersand,
    find_default_branching_context, find_gateway_condition_expression_span,
    find_gateway_condition_expression_text, find_invalid_default_flow_context,
    find_missing_branch_condition_context, find_task_routing_violations,
    find_unescaped_ampersand_span, find_unescaped_placeholder_span, find_xml_error_token_span,
    malformed_closing_tag_line_fix, preferred_default_flow,
    should_append_source_gateway_condition_issues, should_append_source_task_routing_issue,
    should_append_source_unsupported_condition_issues,
    source_duplicate_unconditional_gateway_issues, source_invalid_default_gateway_issues,
    source_issue_group_size, source_task_routing_issue, task_routing_structured_repair,
    task_routing_violations_json, task_routing_violations_summary,
    unsupported_condition_expression_help,
};
use crate::lint::bpmn::{
    ambiguous_boolean_gateway_condition_issues, ambiguous_boolean_gateway_condition_source_issues,
    deferred_document_surface_issue, human_task_interaction_issues, human_task_standard_issues,
    issue_from_bpmn_document_error, issue_from_bpmn_execution_shape_error,
    issue_from_bpmn_human_task_standard_error, issue_from_bpmn_identity_error,
    issue_from_bpmn_reference_error, issue_from_bpmn_topology_error, loop_risk_issues,
    task_operation_binding_issues, undeclared_gateway_condition_output_issues,
    unexpected_bpmn_issue, unsupported_gateway_condition_source_issues,
};
use crate::{
    BpmnEngineError, BpmnPackage, BpmnParseOptions, BpmnSourceFile, LintDomain, LintIssue,
    LintReport, LintSourceDiagnostic, LintSourceSpan, parse_bpmn_package,
};
use serde_json::json;

/// Lints one BPMN source and returns an LLM-friendly blocking report.
#[must_use]
pub(crate) fn lint_bpmn_source_impl(source: &BpmnSourceFile) -> LintReport {
    let pre_parse_interaction_issues =
        human_task_interaction_issues(source, &BpmnPackage::new("", vec![]));
    if !pre_parse_interaction_issues.is_empty() {
        return LintReport::blocking(
            LintDomain::Bpmn,
            &source.source_id,
            pre_parse_interaction_issues,
        );
    }

    if let Some(issue) = deferred_document_surface_issue(source) {
        return LintReport::blocking(LintDomain::Bpmn, &source.source_id, vec![issue]);
    }

    match parse_bpmn_package(std::slice::from_ref(source), &BpmnParseOptions::default()) {
        Ok(package) => {
            let extension_issues = human_task_interaction_issues(source, &package);
            if !extension_issues.is_empty() {
                return LintReport::blocking(LintDomain::Bpmn, &source.source_id, extension_issues);
            }
            let human_task_issues = human_task_standard_issues(source);
            if !human_task_issues.is_empty() {
                return LintReport::blocking(
                    LintDomain::Bpmn,
                    &source.source_id,
                    human_task_issues,
                );
            }
            let operation_binding_issues = task_operation_binding_issues(source);
            if !operation_binding_issues.is_empty() {
                return LintReport::blocking(
                    LintDomain::Bpmn,
                    &source.source_id,
                    operation_binding_issues,
                );
            }
            let data_contract_issues = undeclared_gateway_condition_output_issues(source);
            if !data_contract_issues.is_empty() {
                return LintReport::blocking(
                    LintDomain::Bpmn,
                    &source.source_id,
                    data_contract_issues,
                );
            }
            let condition_issues = ambiguous_boolean_gateway_condition_issues(source, &package);
            if !condition_issues.is_empty() {
                return LintReport::blocking(LintDomain::Bpmn, &source.source_id, condition_issues);
            }
            let loop_issues = loop_risk_issues(source, &package);
            if !loop_issues.is_empty() {
                return LintReport::blocking(LintDomain::Bpmn, &source.source_id, loop_issues);
            }
            LintReport::ok(LintDomain::Bpmn, &source.source_id)
        }
        Err(error) => {
            let mut issues = vec![issue_from_bpmn_error(source, &error)];
            if should_append_source_task_routing_issue(&error)
                && let Some(issue) = source_task_routing_issue(source)
            {
                issues.push(issue);
            }
            if should_append_source_gateway_condition_issues(&error) {
                issues.extend(ambiguous_boolean_gateway_condition_source_issues(source));
                issues.extend(source_duplicate_unconditional_gateway_issues(source));
                issues.extend(source_invalid_default_gateway_issues(source));
            }
            if should_append_source_unsupported_condition_issues(&error) {
                let mut unsupported_condition_issues =
                    unsupported_gateway_condition_source_issues(source);
                if matches!(
                    error,
                    BpmnEngineError::UnsupportedGatewayConfiguration {
                        detail: "unsupported_condition_expression",
                        ..
                    }
                ) {
                    unsupported_condition_issues.retain(|issue| source_issue_group_size(issue) > 1);
                }
                append_unique_source_issues(&mut issues, unsupported_condition_issues);
            }
            LintReport::blocking(LintDomain::Bpmn, &source.source_id, issues)
        }
    }
}

fn issue_from_bpmn_error(source: &BpmnSourceFile, error: &BpmnEngineError) -> LintIssue {
    if let Some(issue) = issue_from_checkpoint_xml_escape_error(source, error) {
        return issue;
    }

    let issue = issue_from_bpmn_human_task_standard_error(source, error)
        .or_else(|| issue_from_bpmn_document_error(error))
        .or_else(|| issue_from_bpmn_identity_error(error))
        .or_else(|| issue_from_bpmn_reference_error(error))
        .or_else(|| issue_from_bpmn_topology_error(error))
        .or_else(|| issue_from_bpmn_execution_shape_error(error))
        .unwrap_or_else(|| unexpected_bpmn_issue(source, error));

    attach_bpmn_source_diagnostic(source, error, issue)
}

fn issue_from_checkpoint_xml_escape_error(
    source: &BpmnSourceFile,
    error: &BpmnEngineError,
) -> Option<LintIssue> {
    let BpmnEngineError::CheckpointCodec(message) = error else {
        return None;
    };
    if !message.contains("Cannot find ';' after '&'")
        && !message.contains("Error while escaping character")
    {
        return None;
    }

    let span = find_unescaped_ampersand_span(&source.contents)?;
    let replacement_line = escaped_line_fix_for_ampersand(&source.contents, span.start)?;
    Some(
        LintIssue::from_parts(
            "bpmn.invalid_xml",
            "BPMN XML is not well-formed",
            format!(
                "Source '{}' contains a raw ampersand in XML text or an attribute.",
                source.source_id
            ),
            "XML text and attribute values must escape literal ampersands as `&amp;`; otherwise the parser treats them as entity references.",
            vec![
                "Replace the raw `&` with `&amp;` in text or attribute values.".to_string(),
                "Do not escape ampersands that already start valid XML entities such as `&amp;`, `&lt;`, or `&#123;`.".to_string(),
            ],
            format!(
                "Repair BPMN source '{}' by replacing the raw ampersand with `&amp;` while preserving BPMN ids and workflow structure.",
                source.source_id
            ),
            json!({
                "source_id": source.source_id,
                "engine_error": message,
            }),
        )
        .with_source_diagnostic(LintSourceDiagnostic::new(
            &source.source_id,
            LintSourceSpan::new(span.start, span.end),
            "escape raw ampersand as `&amp;`",
            "Replace this literal `&` with `&amp;`. Preserve BPMN ids and native metadata.",
        ))
        .with_structured_repair(json!({
            "schema_version": 1,
            "contract": "bpmn.native.xml.well_formed.v1",
            "contract_message": "bpmn.native.xml.well_formed.v1 requires literal ampersands in XML text or attributes to be escaped as &amp;.",
            "strategy": "escape_raw_ampersand",
            "line_fixes": [{
                "offset": span.start,
                "xml": replacement_line
            }]
        })),
    )
}

fn attach_bpmn_source_diagnostic(
    source: &BpmnSourceFile,
    error: &BpmnEngineError,
    issue: LintIssue,
) -> LintIssue {
    if let BpmnEngineError::InvalidXml { offset, .. } = error {
        return attach_invalid_xml_source_diagnostic(source, *offset, issue);
    }
    if let BpmnEngineError::UnsupportedTaskConfiguration {
        node_id, detail, ..
    } = error
        && *detail == "task_requires_single_outgoing"
    {
        return attach_task_requires_single_outgoing(source, node_id, detail, issue);
    }
    if let BpmnEngineError::UnsupportedGatewayConfiguration {
        node_id, detail, ..
    } = error
    {
        return match *detail {
            "default_flow_requires_multiple_outgoing" => {
                attach_default_flow_requires_multiple_outgoing(source, node_id, detail, issue)
            }
            "unknown_default_flow" | "default_flow_not_outgoing" => {
                attach_invalid_default_flow_source_diagnostic(source, node_id, detail, issue)
            }
            "missing_condition_expression" => {
                attach_missing_condition_expression_source_diagnostic(
                    source, node_id, detail, issue,
                )
            }
            "unsupported_condition_expression" => {
                attach_unsupported_condition_expression_source_diagnostic(source, node_id, issue)
            }
            _ => issue,
        };
    }
    issue
}

fn attach_invalid_xml_source_diagnostic(
    source: &BpmnSourceFile,
    offset: Option<u64>,
    issue: LintIssue,
) -> LintIssue {
    if let Some((span, tag_name)) = find_unescaped_placeholder_span(&source.contents, offset) {
        return issue
            .with_source_diagnostic(LintSourceDiagnostic::new(
                &source.source_id,
                LintSourceSpan::new(span.start, span.end),
                format!("escape raw XML-like placeholder `<{tag_name}>` in text"),
                format!(
                    "Replace `<{tag_name}>` with `&lt;{tag_name}&gt;` inside prompt/question text, or wrap literal examples in CDATA. Preserve BPMN ids."
                ),
            ))
            .with_structured_repair(json!({
                "schema_version": 1,
                "contract": "bpmn.native.xml.well_formed.v1",
                "strategy": "escape_unescaped_xml_text_placeholder",
                "actions": [{
                    "op": "escape_text_node_placeholder",
                    "target": format!("<{tag_name}>"),
                    "examples": [
                        format!("&lt;{tag_name}&gt;"),
                        format!("<![CDATA[<{tag_name}>]]>")
                    ],
                    "forbidden_forms": [format!("<{tag_name}>")]
                }]
            }));
    }
    let Some(span) = find_xml_error_token_span(&source.contents, offset) else {
        return issue;
    };
    let structured_repair = if let Some(replacement_line) =
        malformed_closing_tag_line_fix(&source.contents, span.start)
    {
        json!({
            "schema_version": 1,
            "contract": "bpmn.native.xml.well_formed.v1",
            "contract_message": "bpmn.native.xml.well_formed.v1 requires opening and closing XML element names to match exactly.",
            "strategy": "repair_malformed_xml_closing_tag",
            "line_fixes": [{
                "offset": span.start,
                "xml": replacement_line
            }]
        })
    } else {
        json!({
            "schema_version": 1,
            "contract": "bpmn.native.xml.well_formed.v1",
            "strategy": "repair_malformed_xml_token",
            "actions": [{
                "op": "repair_xml_tag_or_nesting",
                "target": "parser offset token",
                "forbidden_forms": [
                    "escaping real BPMN elements such as &lt;extensionElements&gt;",
                    "changing existing BPMN ids while repairing XML syntax"
                ]
            }]
        })
    };
    issue
        .with_source_diagnostic(LintSourceDiagnostic::new(
            &source.source_id,
            LintSourceSpan::new(span.start, span.end),
            "repair malformed XML near this token",
            "Fix tag spelling, namespace prefix spelling, closing tags, attribute quotes, or nesting. Do not escape real BPMN element tags.",
        ))
        .with_structured_repair(structured_repair)
}

fn attach_default_flow_requires_multiple_outgoing(
    source: &BpmnSourceFile,
    node_id: &str,
    detail: &str,
    issue: LintIssue,
) -> LintIssue {
    let Some(context) = find_default_branching_context(&source.contents, node_id) else {
        return issue;
    };
    let outgoing_flow_ids = context
        .outgoing_flows
        .iter()
        .map(|flow| flow.id.clone())
        .collect::<Vec<_>>();
    let outgoing_summary = if outgoing_flow_ids.is_empty() {
        "none".to_string()
    } else {
        outgoing_flow_ids.join(", ")
    };
    let mut issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(context.gateway_span.start, context.gateway_span.end),
        format!("default=\"{}\" needs a real fallback branch", context.default_flow_id),
        format!(
            "Gateway `{node_id}` currently has {} outgoing flow(s): {outgoing_summary}. Remove the default attribute when there is no fallback, or add one unconditional fallback sequenceFlow from this gateway and point default at it.",
            outgoing_flow_ids.len(),
        ),
    ));
    issue.evidence = json!({
        "gateway_id": node_id,
        "default_flow_id": context.default_flow_id,
        "outgoing_flow_ids": outgoing_flow_ids,
        "outgoing_flow_count": context.outgoing_flows.len(),
        "detail": detail,
    });
    issue
}

fn attach_task_requires_single_outgoing(
    source: &BpmnSourceFile,
    node_id: &str,
    detail: &str,
    issue: LintIssue,
) -> LintIssue {
    let violations = find_task_routing_violations(&source.contents);
    let Some(primary) = violations
        .iter()
        .find(|violation| violation.task_id.as_str() == node_id)
        .or_else(|| violations.first())
    else {
        return issue;
    };
    let outgoing_flow_ids = primary
        .outgoing_flows
        .iter()
        .map(|flow| flow.id.clone())
        .collect::<Vec<_>>();
    let outgoing_summary = if outgoing_flow_ids.is_empty() {
        "none".to_string()
    } else {
        outgoing_flow_ids.join(", ")
    };
    let all_violations_summary = task_routing_violations_summary(&violations);
    let mut issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(primary.task_span.start, primary.task_span.end),
        "task must have exactly one outgoing sequenceFlow",
        format!(
            "Task `{}` currently has {} outgoing flow(s): {outgoing_summary}. Lint detail: {detail}. Add one route to the next node, or route to one gateway if branching is required. All task route violations: {all_violations_summary}.",
            primary.task_id,
            outgoing_flow_ids.len(),
        ),
    ));
    issue.repair_guidance.insert(
        0,
        format!(
            "Repair every task route violation in the same patch, not only `{}`: {all_violations_summary}.",
            primary.task_id
        ),
    );
    issue.llm_fix_prompt = format!(
        "Repair all executable task route violations in '{}': {all_violations_summary}. Do not stop after `{}`; every listed task must end with exactly one outgoing sequenceFlow. Preserve task ids and host configs.",
        source.source_id, primary.task_id
    );
    issue.evidence = json!({
        "task_id": primary.task_id.clone(),
        "outgoing_flow_ids": outgoing_flow_ids.clone(),
        "outgoing_flow_count": primary.outgoing_flows.len(),
        "all_task_route_violations": task_routing_violations_json(&violations),
        "detail": detail,
    });
    issue.structured_repair = Some(task_routing_structured_repair(&violations));
    issue
}

fn attach_invalid_default_flow_source_diagnostic(
    source: &BpmnSourceFile,
    node_id: &str,
    detail: &str,
    issue: LintIssue,
) -> LintIssue {
    let Some(context) = find_invalid_default_flow_context(&source.contents, node_id) else {
        return issue;
    };
    let valid_flow_ids = context
        .outgoing_flows
        .iter()
        .map(|flow| flow.id.clone())
        .collect::<Vec<_>>();
    let preferred_flow = preferred_default_flow(&context);
    let preferred_flow_id = preferred_flow.map(|flow| flow.id.clone());
    let preferred_has_condition = preferred_flow.is_some_and(|flow| flow.has_condition);
    let valid_list = valid_flow_ids.join(", ");
    let preferred = preferred_flow_id
        .as_deref()
        .unwrap_or("one existing outgoing sequenceFlow id from this gateway");
    let condition_help = if preferred_has_condition {
        " Remove that sequenceFlow conditionExpression when making it the default branch."
    } else {
        ""
    };
    let mut issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(context.gateway_span.start, context.gateway_span.end),
        format!("retarget stale default flow `{}`", context.default_flow_id),
        format!(
            "Valid outgoing flow ids from gateway `{node_id}`: {valid_list}. Prefer `default=\"{preferred}\"` when it is the intended fallback.{condition_help}"
        ),
    ));
    issue.repair_guidance = invalid_default_repair_guidance(
        node_id,
        &context.default_flow_id,
        &valid_list,
        preferred,
        preferred_has_condition,
    );
    issue.llm_fix_prompt = format!(
        "Repair gateway '{node_id}' by changing its `default` attribute from stale flow id '{}' to one existing outgoing sequenceFlow id from the same gateway. Valid ids: {valid_list}. Prefer `{preferred}` if it is the fallback branch, and remove that branch conditionExpression if present. Return a minimal XML diff only.",
        context.default_flow_id
    );
    issue.evidence = json!({
        "process_node_id": node_id,
        "invalid_default_flow_id": context.default_flow_id.clone(),
        "valid_outgoing_flow_ids": valid_flow_ids.clone(),
        "preferred_default_flow_id": preferred_flow_id.clone(),
        "preferred_default_has_condition": preferred_has_condition,
        "detail": detail,
    });
    issue.structured_repair = Some(invalid_default_structured_repair(
        node_id,
        &context,
        &valid_flow_ids,
        preferred_flow_id.as_ref(),
        preferred_has_condition,
    ));
    issue
}

fn invalid_default_repair_guidance(
    node_id: &str,
    default_flow_id: &str,
    valid_list: &str,
    preferred: &str,
    preferred_has_condition: bool,
) -> Vec<String> {
    vec![
        format!(
            "Replace `default=\"{default_flow_id}\"` on gateway '{node_id}' with one valid outgoing flow id from this same gateway: {valid_list}."
        ),
        format!(
            "Prefer `default=\"{preferred}\"` when it matches a stale renamed default branch or the intended fallback."
        ),
        if preferred_has_condition {
            format!(
                "Remove the `conditionExpression` from sequenceFlow '{preferred}' if you select it as the gateway default."
            )
        } else {
            "Keep the selected default branch unconditional; conditions belong only on non-default outgoing branches.".to_string()
        },
        "Do not point a gateway `default` at a missing flow id, a renamed stale id, or a sequenceFlow whose sourceRef is a different node.".to_string(),
    ]
}

fn invalid_default_structured_repair(
    node_id: &str,
    context: &InvalidDefaultFlowContext,
    valid_flow_ids: &[String],
    preferred_flow_id: Option<&String>,
    preferred_has_condition: bool,
) -> serde_json::Value {
    json!({
        "schema_version": 1,
        "contract": "bpmn.native.gateway.bounded.v1",
        "strategy": "retarget_default_flow_to_existing_outgoing",
        "actions": [{
            "op": "set_gateway_default",
            "target": format!("gateway#{node_id}"),
            "current": context.default_flow_id.clone(),
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
    })
}

fn attach_missing_condition_expression_source_diagnostic(
    source: &BpmnSourceFile,
    node_id: &str,
    detail: &str,
    issue: LintIssue,
) -> LintIssue {
    let Some(context) = find_missing_branch_condition_context(&source.contents, node_id) else {
        return issue;
    };
    let flow_id = context.flow_id.clone();
    let target_ref = context.target_ref.clone();
    let duplicate_conditioned_flow_ids = context.duplicate_conditioned_flow_ids.clone();
    let duplicate_default_flow_ids = context.duplicate_default_flow_ids.clone();
    let duplicate_kind =
        missing_branch_duplicate_kind(&duplicate_conditioned_flow_ids, &duplicate_default_flow_ids);
    let duplicate_ids = match duplicate_kind {
        MissingBranchDuplicateKind::Conditioned => duplicate_conditioned_flow_ids.join(", "),
        MissingBranchDuplicateKind::Default => duplicate_default_flow_ids.join(", "),
        MissingBranchDuplicateKind::None => String::new(),
    };
    let mut issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(context.flow_span.start, context.flow_span.end),
        missing_condition_label(duplicate_kind),
        missing_condition_help(duplicate_kind, &duplicate_ids, &flow_id),
    ));
    if duplicate_kind == MissingBranchDuplicateKind::None {
        issue.repair_guidance.insert(
            0,
            format!("Repair sequenceFlow '{flow_id}' from gateway '{node_id}'."),
        );
    } else {
        issue.repair_guidance =
            duplicate_unconditional_repair_guidance(&flow_id, &duplicate_ids, duplicate_kind);
    }
    issue.llm_fix_prompt =
        missing_condition_llm_prompt(node_id, &flow_id, &duplicate_ids, duplicate_kind);
    issue.evidence = json!({
        "gateway_id": node_id,
        "missing_condition_flow_id": flow_id.clone(),
        "target_ref": target_ref.clone(),
        "duplicate_conditioned_flow_ids": duplicate_conditioned_flow_ids.clone(),
        "duplicate_default_flow_ids": duplicate_default_flow_ids.clone(),
        "detail": detail,
    });
    issue.structured_repair = Some(missing_condition_structured_repair(
        node_id,
        &flow_id,
        target_ref.as_ref(),
        &duplicate_conditioned_flow_ids,
        &duplicate_default_flow_ids,
        duplicate_kind,
    ));
    issue
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MissingBranchDuplicateKind {
    None,
    Conditioned,
    Default,
}

pub(super) fn missing_branch_duplicate_kind(
    duplicate_conditioned_flow_ids: &[String],
    duplicate_default_flow_ids: &[String],
) -> MissingBranchDuplicateKind {
    if !duplicate_conditioned_flow_ids.is_empty() {
        MissingBranchDuplicateKind::Conditioned
    } else if !duplicate_default_flow_ids.is_empty() {
        MissingBranchDuplicateKind::Default
    } else {
        MissingBranchDuplicateKind::None
    }
}

pub(super) fn missing_condition_label(duplicate_kind: MissingBranchDuplicateKind) -> &'static str {
    match duplicate_kind {
        MissingBranchDuplicateKind::Conditioned => "remove this duplicate unconditional branch",
        MissingBranchDuplicateKind::Default => "remove this duplicate fallback branch",
        MissingBranchDuplicateKind::None => "condition or make this branch the gateway default",
    }
}

pub(super) fn missing_condition_help(
    duplicate_kind: MissingBranchDuplicateKind,
    duplicate_ids: &str,
    flow_id: &str,
) -> String {
    match duplicate_kind {
        MissingBranchDuplicateKind::Conditioned => format!(
            "This unconditional branch duplicates conditioned branch(es): {duplicate_ids}. Remove `{flow_id}`; do not add a condition or make it the default."
        ),
        MissingBranchDuplicateKind::Default => format!(
            "This unconditional branch duplicates default fallback branch(es): {duplicate_ids}. Remove `{flow_id}`; keep the existing gateway default branch."
        ),
        MissingBranchDuplicateKind::None => "Add a child `conditionExpression` with one allowed form such as `approved` or `not approved`, or if this branch is the fallback, set the gateway `default` attribute to this sequenceFlow id and leave it unconditional.".to_string(),
    }
}

pub(super) fn duplicate_unconditional_repair_guidance(
    flow_id: &str,
    duplicate_ids: &str,
    duplicate_kind: MissingBranchDuplicateKind,
) -> Vec<String> {
    match duplicate_kind {
        MissingBranchDuplicateKind::Conditioned => vec![
            format!(
                "Remove duplicate unconditional sequenceFlow '{flow_id}' because it has the same sourceRef/targetRef as conditioned branch(es): {duplicate_ids}."
            ),
            "Keep the conditioned branch and the gateway default fallback; do not add a new condition to the duplicate branch.".to_string(),
            "Do not make the duplicate branch the default unless the conditioned branch is removed and fallback intent changes.".to_string(),
        ],
        MissingBranchDuplicateKind::Default => vec![
            format!(
                "Remove duplicate unconditional sequenceFlow '{flow_id}' because it has the same sourceRef/targetRef as default fallback branch(es): {duplicate_ids}."
            ),
            "Keep the existing gateway default fallback branch; do not add a condition to the duplicate branch.".to_string(),
            "Do not change the gateway default when deleting the duplicate fallback branch.".to_string(),
        ],
        MissingBranchDuplicateKind::None => Vec::new(),
    }
}

pub(super) fn missing_condition_llm_prompt(
    node_id: &str,
    flow_id: &str,
    duplicate_ids: &str,
    duplicate_kind: MissingBranchDuplicateKind,
) -> String {
    match duplicate_kind {
        MissingBranchDuplicateKind::Conditioned => format!(
            "Repair gateway '{node_id}' by deleting duplicate unconditional sequenceFlow '{flow_id}'. It duplicates conditioned branch(es) {duplicate_ids}. Do not add a condition to '{flow_id}' and do not make it the default. Return a minimal XML diff only."
        ),
        MissingBranchDuplicateKind::Default => format!(
            "Repair gateway '{node_id}' by deleting duplicate unconditional sequenceFlow '{flow_id}'. It duplicates default fallback branch(es) {duplicate_ids}. Do not add a condition to '{flow_id}' and do not change the gateway default. Return a minimal XML diff only."
        ),
        MissingBranchDuplicateKind::None => format!(
            "Repair gateway '{node_id}' by either adding one bounded conditionExpression to non-default sequenceFlow '{flow_id}', or by making that sequenceFlow the gateway default if it is the fallback. Preserve workflow intent."
        ),
    }
}

pub(super) fn missing_condition_structured_repair(
    node_id: &str,
    flow_id: &str,
    target_ref: Option<&String>,
    duplicate_conditioned_flow_ids: &[String],
    duplicate_default_flow_ids: &[String],
    duplicate_kind: MissingBranchDuplicateKind,
) -> serde_json::Value {
    if duplicate_kind == MissingBranchDuplicateKind::None {
        json!({
            "schema_version": 1,
            "contract": "bpmn.native.gateway.bounded.v1",
            "strategy": "resolve_unconditional_non_default_branch",
            "target": {
                "gateway_id": node_id,
                "sequence_flow_id": flow_id,
                "target_ref": target_ref,
                "duplicate_conditioned_flow_ids": duplicate_conditioned_flow_ids,
                "duplicate_default_flow_ids": duplicate_default_flow_ids
            },
            "actions": [{
                "op": "choose_one",
                "allowed_forms": ["boolean_path", "numeric_comparison"],
                "examples": [
                    "<conditionExpression xsi:type=\"tFormalExpression\">approved</conditionExpression>",
                    format!("default=\"{flow_id}\"")
                ],
                "options": [
                    {
                        "op": "add_condition_expression_to_non_default_branch",
                        "target": flow_id,
                        "allowed_forms": ["boolean_path", "numeric_comparison"]
                    },
                    {
                        "op": "promote_unconditional_branch_to_default",
                        "target": flow_id,
                        "when": "the branch is the fallback route",
                        "requires": "set the gateway default attribute to this sequenceFlow id and keep that flow without conditionExpression"
                    }
                ],
                "forbidden_forms": [
                    "conditionExpression on the default branch",
                    "missing conditionExpression on a non-default branch"
                ]
            }]
        })
    } else {
        json!({
            "schema_version": 1,
            "contract": "bpmn.native.gateway.bounded.v1",
            "strategy": "remove_duplicate_unconditional_gateway_branch",
            "target": {
                "gateway_id": node_id,
                "sequence_flow_id": flow_id,
                "target_ref": target_ref,
                "duplicate_conditioned_flow_ids": duplicate_conditioned_flow_ids,
                "duplicate_default_flow_ids": duplicate_default_flow_ids
            },
            "actions": [{
                "op": "remove_duplicate_unconditional_flow",
                "target": flow_id,
                "forbid": [
                    "adding conditionExpression to the duplicate",
                    "making the duplicate branch default",
                    "leaving duplicate sourceRef/targetRef branches"
                ]
            }]
        })
    }
}

fn attach_unsupported_condition_expression_source_diagnostic(
    source: &BpmnSourceFile,
    node_id: &str,
    issue: LintIssue,
) -> LintIssue {
    let Some(span) = find_gateway_condition_expression_span(&source.contents, node_id) else {
        return issue;
    };
    let condition = find_gateway_condition_expression_text(&source.contents, node_id)
        .unwrap_or_else(|| {
            source
                .contents
                .get(span.clone())
                .unwrap_or_default()
                .to_string()
        });
    issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        "rewrite this condition into the bounded native subset",
        unsupported_condition_expression_help(&condition),
    ))
}
