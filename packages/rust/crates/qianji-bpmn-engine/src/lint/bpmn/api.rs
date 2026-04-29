use super::condition_contract::{
    ambiguous_boolean_gateway_condition_issues, ambiguous_boolean_gateway_condition_source_issues,
    unsupported_gateway_condition_source_issues,
};
use super::data_contract::undeclared_gateway_condition_output_issues;
use super::document::issue_from_bpmn_document_error;
use super::document_surface::deferred_document_surface_issue;
use super::execution::issue_from_bpmn_execution_shape_error;
use super::extension::human_task_interaction_issues;
use super::human_task::{human_task_standard_issues, issue_from_bpmn_human_task_standard_error};
use super::identity::issue_from_bpmn_identity_error;
use super::loop_risk::loop_risk_issues;
use super::reference::issue_from_bpmn_reference_error;
use super::topology::issue_from_bpmn_topology_error;
use super::unexpected::unexpected_bpmn_issue;
use crate::bpmn_parse_api::{BpmnParseOptions, BpmnSourceFile, parse_bpmn_package};
use crate::error::BpmnEngineError;
use crate::lint_api::{LintDomain, LintIssue, LintReport, LintSourceDiagnostic, LintSourceSpan};
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use serde_json::json;
use std::borrow::Cow;

/// Lints one BPMN source and returns an LLM-friendly blocking report.
#[must_use]
pub(crate) fn lint_bpmn_source_impl(source: &BpmnSourceFile) -> LintReport {
    let pre_parse_interaction_issues =
        human_task_interaction_issues(source, &crate::ir_package_api::BpmnPackage::new("", vec![]));
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
        LintIssue::new(
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
enum MissingBranchDuplicateKind {
    None,
    Conditioned,
    Default,
}

fn missing_branch_duplicate_kind(
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

fn missing_condition_label(duplicate_kind: MissingBranchDuplicateKind) -> &'static str {
    match duplicate_kind {
        MissingBranchDuplicateKind::Conditioned => "remove this duplicate unconditional branch",
        MissingBranchDuplicateKind::Default => "remove this duplicate fallback branch",
        MissingBranchDuplicateKind::None => "condition or make this branch the gateway default",
    }
}

fn missing_condition_help(
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

fn duplicate_unconditional_repair_guidance(
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

fn missing_condition_llm_prompt(
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

fn missing_condition_structured_repair(
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

struct InvalidDefaultFlowContext {
    default_flow_id: String,
    gateway_span: std::ops::Range<usize>,
    outgoing_flows: Vec<OutgoingFlowSummary>,
}

struct DefaultBranchingContext {
    default_flow_id: String,
    gateway_span: std::ops::Range<usize>,
    outgoing_flows: Vec<OutgoingFlowSummary>,
}

struct TaskRoutingViolation {
    task_id: String,
    task_span: std::ops::Range<usize>,
    outgoing_flows: Vec<OutgoingFlowSummary>,
}

#[derive(Clone)]
struct OutgoingFlowSummary {
    id: String,
    has_condition: bool,
}

struct MissingBranchConditionContext {
    flow_id: String,
    target_ref: Option<String>,
    flow_span: std::ops::Range<usize>,
    duplicate_conditioned_flow_ids: Vec<String>,
    duplicate_default_flow_ids: Vec<String>,
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

fn preferred_default_flow(context: &InvalidDefaultFlowContext) -> Option<&OutgoingFlowSummary> {
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

fn find_invalid_default_flow_context(
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

fn find_default_branching_context(
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

fn should_append_source_task_routing_issue(error: &BpmnEngineError) -> bool {
    matches!(error, BpmnEngineError::UnknownSequenceFlowEndpoint { .. })
}

fn should_append_source_gateway_condition_issues(error: &BpmnEngineError) -> bool {
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

fn should_append_source_unsupported_condition_issues(error: &BpmnEngineError) -> bool {
    !matches!(error, BpmnEngineError::InvalidXml { .. })
}

fn append_unique_source_issues(issues: &mut Vec<LintIssue>, candidates: Vec<LintIssue>) {
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

fn source_issue_group_size(issue: &LintIssue) -> usize {
    issue
        .evidence
        .get("conditions")
        .and_then(|value| value.as_array())
        .map_or(1, Vec::len)
}

fn source_duplicate_unconditional_gateway_issues(source: &BpmnSourceFile) -> Vec<LintIssue> {
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

fn source_invalid_default_gateway_issues(source: &BpmnSourceFile) -> Vec<LintIssue> {
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

fn source_task_routing_issue(source: &BpmnSourceFile) -> Option<LintIssue> {
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

fn find_task_routing_violations(contents: &str) -> Vec<TaskRoutingViolation> {
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

fn task_routing_violations_summary(violations: &[TaskRoutingViolation]) -> String {
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

fn task_routing_violations_json(violations: &[TaskRoutingViolation]) -> serde_json::Value {
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

fn task_routing_structured_repair(violations: &[TaskRoutingViolation]) -> serde_json::Value {
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

fn find_unescaped_placeholder_span(
    contents: &str,
    parser_offset: Option<u64>,
) -> Option<(std::ops::Range<usize>, String)> {
    let mut cursor = usize::try_from(parser_offset?)
        .ok()?
        .min(contents.len().saturating_sub(1));
    loop {
        let start = contents.get(..=cursor)?.rfind('<')?;
        if let Some(placeholder) = xml_placeholder_tag_at(contents, start) {
            return Some(placeholder);
        }
        if start == 0 {
            return None;
        }
        cursor = start - 1;
    }
}

fn xml_placeholder_tag_at(
    contents: &str,
    start: usize,
) -> Option<(std::ops::Range<usize>, String)> {
    let end = contents.get(start..)?.find('>')? + start + 1;
    let tag_name = contents
        .get(start + 1..end - 1)?
        .trim()
        .trim_end_matches('/');
    if tag_name.is_empty()
        || tag_name.starts_with(['/', '?', '!'])
        || tag_name.contains(char::is_whitespace)
        || tag_name.contains(':')
        || is_known_xml_element_hint(tag_name)
        || !tag_name.bytes().all(is_xml_name_hint_byte)
    {
        return None;
    }
    Some((start..end, tag_name.to_string()))
}

fn is_known_xml_element_hint(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "definitions"
            | "process"
            | "documentation"
            | "extensionElements"
            | "startEvent"
            | "endEvent"
            | "intermediateCatchEvent"
            | "intermediateThrowEvent"
            | "serviceTask"
            | "userTask"
            | "manualTask"
            | "businessRuleTask"
            | "scriptTask"
            | "receiveTask"
            | "sendTask"
            | "exclusiveGateway"
            | "inclusiveGateway"
            | "parallelGateway"
            | "eventBasedGateway"
            | "sequenceFlow"
            | "conditionExpression"
            | "boundaryEvent"
            | "subProcess"
            | "transaction"
            | "callActivity"
            | "errorEventDefinition"
            | "messageEventDefinition"
            | "signalEventDefinition"
            | "timerEventDefinition"
            | "cancelEventDefinition"
            | "compensateEventDefinition"
            | "script"
            | "standardLoopCharacteristics"
            | "multiInstanceLoopCharacteristics"
            | "loopCardinality"
            | "completionCondition"
            | "loopDataInputRef"
            | "loopDataOutputRef"
            | "inputDataItem"
            | "outputDataItem"
            | "association"
            | "laneSet"
            | "lane"
            | "flowNodeRef"
    )
}

fn is_xml_name_hint_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':' | b'.')
}

fn find_xml_error_token_span(
    contents: &str,
    parser_offset: Option<u64>,
) -> Option<std::ops::Range<usize>> {
    let offset = usize::try_from(parser_offset?)
        .ok()?
        .min(contents.len().saturating_sub(1));
    let start = contents
        .get(..=offset)?
        .rfind('<')
        .filter(|start| offset.saturating_sub(*start) <= 160)
        .unwrap_or(offset);
    let end = contents
        .get(offset..)?
        .find('>')
        .map(|delta| offset + delta + 1)
        .filter(|end| end.saturating_sub(start) <= 200)
        .unwrap_or_else(|| (offset + 1).min(contents.len()));
    (start < end).then_some(start..end)
}

fn find_unescaped_ampersand_span(contents: &str) -> Option<std::ops::Range<usize>> {
    let mut cursor = 0usize;
    while cursor < contents.len() {
        if contents.get(cursor..)?.starts_with("<!--") {
            cursor = contents
                .get(cursor + 4..)?
                .find("-->")
                .map_or(contents.len(), |offset| cursor + 4 + offset + 3);
            continue;
        }
        if contents.get(cursor..)?.starts_with("<![CDATA[") {
            cursor = contents
                .get(cursor + 9..)?
                .find("]]>")
                .map_or(contents.len(), |offset| cursor + 9 + offset + 3);
            continue;
        }
        if contents.as_bytes().get(cursor) == Some(&b'&')
            && !is_valid_xml_entity_at(contents, cursor)
        {
            return Some(cursor..cursor + 1);
        }
        cursor += contents
            .get(cursor..)?
            .chars()
            .next()
            .map_or(1, char::len_utf8);
    }
    None
}

fn escaped_line_fix_for_ampersand(contents: &str, offset: usize) -> Option<String> {
    let (line_start, line_end) = line_bounds_for_offset(contents, offset)?;
    let line = contents.get(line_start..line_end)?;
    Some(escape_unescaped_ampersands(line.trim_start()))
}

fn malformed_closing_tag_line_fix(contents: &str, token_offset: usize) -> Option<String> {
    let (line_start, line_end) = line_bounds_for_offset(contents, token_offset)?;
    let line = contents.get(line_start..line_end)?;
    let relative_offset = token_offset.checked_sub(line_start)?;
    let token_start = line.get(..=relative_offset)?.rfind('<')?;
    let token_end = line.get(token_start..)?.find('>')? + token_start + 1;
    let closing_tag = line.get(token_start..token_end)?;
    let closing_name = closing_tag_name(closing_tag)?;
    let closing_local_name = xml_local_name(closing_name);
    let opening_name = find_opening_name_for_local(line, token_start, closing_local_name)?;
    if opening_name == closing_name {
        return None;
    }

    let mut repaired = String::with_capacity(line.len() + opening_name.len());
    repaired.push_str(line.get(..token_start)?);
    repaired.push_str("</");
    repaired.push_str(&opening_name);
    repaired.push('>');
    repaired.push_str(line.get(token_end..)?);
    Some(repaired.trim_start().to_string())
}

fn closing_tag_name(tag: &str) -> Option<&str> {
    let name = tag.strip_prefix("</")?.strip_suffix('>')?.trim();
    if name.is_empty() || name.contains(char::is_whitespace) {
        return None;
    }
    Some(name)
}

fn find_opening_name_for_local(
    line: &str,
    before_offset: usize,
    local_name: &str,
) -> Option<String> {
    let mut cursor = 0usize;
    let mut matched = None;
    while cursor < before_offset {
        let Some(relative_start) = line.get(cursor..before_offset)?.find('<') else {
            break;
        };
        let start = relative_start + cursor;
        if line.get(start..).is_some_and(|text| {
            text.starts_with("</") || text.starts_with("<!") || text.starts_with("<?")
        }) {
            cursor = start + 1;
            continue;
        }
        let Some(end) = line
            .get(start..before_offset)?
            .find('>')
            .map(|offset| start + offset + 1)
        else {
            break;
        };
        if let Some(name) = opening_tag_name(line.get(start..end)?)
            && xml_local_name(name) == local_name
        {
            matched = Some(name.to_string());
        }
        cursor = end;
    }
    matched
}

fn opening_tag_name(tag: &str) -> Option<&str> {
    let body = tag.strip_prefix('<')?.trim_start();
    if body.starts_with(['/', '!', '?']) {
        return None;
    }
    let end = body
        .find(|character: char| character.is_whitespace() || character == '/' || character == '>')
        .unwrap_or(body.len());
    let name = body.get(..end)?;
    (!name.is_empty()).then_some(name)
}

fn xml_local_name(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_prefix, local)| local)
}

fn line_bounds_for_offset(contents: &str, offset: usize) -> Option<(usize, usize)> {
    if contents.is_empty() {
        return None;
    }
    let offset = offset.min(contents.len().saturating_sub(1));
    let line_start = contents
        .as_bytes()
        .get(..=offset)?
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |position| position + 1);
    let line_end = contents
        .as_bytes()
        .get(offset..)?
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(contents.len(), |position| offset + position);
    Some((line_start, line_end))
}

fn escape_unescaped_ampersands(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    let mut cursor = 0usize;
    while cursor < text.len() {
        if text.as_bytes().get(cursor) == Some(&b'&') && !is_valid_xml_entity_at(text, cursor) {
            escaped.push_str("&amp;");
            cursor += 1;
            continue;
        }
        let Some(character) = text.get(cursor..).and_then(|value| value.chars().next()) else {
            break;
        };
        escaped.push(character);
        cursor += character.len_utf8();
    }
    escaped
}

fn is_valid_xml_entity_at(text: &str, ampersand_offset: usize) -> bool {
    let Some(rest) = text.get(ampersand_offset + 1..) else {
        return false;
    };
    if let Some((entity, _tail)) = rest.split_once(';')
        && resolve_predefined_entity(entity).is_some()
    {
        return true;
    }
    if let Some(hex) = rest
        .strip_prefix("#x")
        .and_then(|value| value.split_once(';'))
    {
        return !hex.0.is_empty() && hex.0.bytes().all(|byte| byte.is_ascii_hexdigit());
    }
    if let Some(decimal) = rest
        .strip_prefix('#')
        .and_then(|value| value.split_once(';'))
    {
        return !decimal.0.is_empty() && decimal.0.bytes().all(|byte| byte.is_ascii_digit());
    }
    false
}

fn find_missing_branch_condition_context(
    contents: &str,
    gateway_id: &str,
) -> Option<MissingBranchConditionContext> {
    let default_flow_id = find_gateway_default_flow_id(contents, gateway_id)?;
    let flows = find_gateway_flow_details(contents, gateway_id, &default_flow_id);
    let missing = flows
        .iter()
        .find(|flow| !flow.is_default && !flow.has_condition)?;
    let duplicate_conditioned_flow_ids = flows
        .iter()
        .filter(|flow| {
            flow.id != missing.id && flow.has_condition && flow.target_ref == missing.target_ref
        })
        .map(|flow| flow.id.clone())
        .collect::<Vec<_>>();
    let duplicate_default_flow_ids = flows
        .iter()
        .filter(|flow| {
            flow.id != missing.id && flow.is_default && flow.target_ref == missing.target_ref
        })
        .map(|flow| flow.id.clone())
        .collect::<Vec<_>>();
    Some(MissingBranchConditionContext {
        flow_id: missing.id.clone(),
        target_ref: missing.target_ref.clone(),
        flow_span: missing.span.clone(),
        duplicate_conditioned_flow_ids,
        duplicate_default_flow_ids,
    })
}

fn find_gateway_flow_details(
    contents: &str,
    gateway_id: &str,
    default_flow_id: &str,
) -> Vec<GatewayFlowDetail> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut active_flow: Option<ActiveGatewayFlow> = None;
    let mut flows = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "sequenceFlow") {
                    let source_ref = attribute_value(&reader, &event, "sourceRef");
                    let flow_id = attribute_value(&reader, &event, "id");
                    if source_ref.as_deref() == Some(gateway_id)
                        && let Some(flow_id) = flow_id
                        && let Some(event_end) = reader_position(&reader)
                        && let Some(span) = start_event_span(event_end, &event)
                    {
                        active_flow = Some(ActiveGatewayFlow {
                            depth,
                            id: flow_id.clone(),
                            target_ref: attribute_value(&reader, &event, "targetRef"),
                            span,
                            has_condition: false,
                            is_default: flow_id == default_flow_id,
                        });
                    }
                } else if active_flow.is_some()
                    && is_element(&event, "conditionExpression")
                    && let Some(flow) = active_flow.as_mut()
                {
                    flow.has_condition = true;
                }
            }
            Ok(Event::Empty(event)) => {
                if is_element(&event, "sequenceFlow") {
                    let source_ref = attribute_value(&reader, &event, "sourceRef");
                    let flow_id = attribute_value(&reader, &event, "id");
                    if source_ref.as_deref() == Some(gateway_id)
                        && let Some(flow_id) = flow_id
                        && let Some(event_end) = reader_position(&reader)
                        && let Some(span) = start_event_span(event_end, &event)
                    {
                        flows.push(GatewayFlowDetail {
                            is_default: flow_id == default_flow_id,
                            id: flow_id,
                            target_ref: attribute_value(&reader, &event, "targetRef"),
                            span,
                            has_condition: false,
                        });
                    }
                } else if active_flow.is_some()
                    && is_element(&event, "conditionExpression")
                    && let Some(flow) = active_flow.as_mut()
                {
                    flow.has_condition = true;
                }
            }
            Ok(Event::End(event)) => {
                if local_name(event.name().as_ref()) == "sequenceFlow"
                    && let Some(flow) = active_flow.take()
                {
                    let ActiveGatewayFlow {
                        depth: _flow_depth,
                        id,
                        target_ref,
                        span,
                        has_condition,
                        is_default,
                    } = flow;
                    flows.push(GatewayFlowDetail {
                        id,
                        target_ref,
                        span,
                        has_condition,
                        is_default,
                    });
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return flows,
            Ok(_) => {}
        }
    }
}

fn find_gateway_default_flow_id(contents: &str, gateway_id: &str) -> Option<String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event))
                if is_element(&event, "exclusiveGateway")
                    || is_element(&event, "inclusiveGateway") =>
            {
                if attribute_value(&reader, &event, "id").as_deref() == Some(gateway_id) {
                    return attribute_value(&reader, &event, "default");
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn find_bounded_gateway_ids(contents: &str) -> Vec<String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut ids = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event))
                if is_element(&event, "exclusiveGateway")
                    || is_element(&event, "inclusiveGateway") =>
            {
                if attribute_value(&reader, &event, "default").is_some()
                    && let Some(id) = attribute_value(&reader, &event, "id")
                {
                    ids.push(id);
                }
            }
            Ok(Event::Eof) | Err(_) => return ids,
            Ok(_) => {}
        }
    }
}

fn find_gateway_default_span_and_id(
    contents: &str,
    gateway_id: &str,
) -> Option<(std::ops::Range<usize>, String)> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event))
                if is_element(&event, "exclusiveGateway")
                    || is_element(&event, "inclusiveGateway") =>
            {
                if attribute_value(&reader, &event, "id").as_deref() == Some(gateway_id) {
                    let default_flow_id = attribute_value(&reader, &event, "default")?;
                    let span = start_event_span(reader_position(&reader)?, &event)?;
                    return Some((span, default_flow_id));
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn find_routable_task_spans(contents: &str) -> Vec<(String, std::ops::Range<usize>)> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut spans = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event)) if is_routable_task_element(&event) => {
                if attribute_value(&reader, &event, "isForCompensation").as_deref() == Some("true")
                {
                    continue;
                }
                if let Some(task_id) = attribute_value(&reader, &event, "id")
                    && let Some(event_end) = reader_position(&reader)
                    && let Some(span) = start_event_span(event_end, &event)
                {
                    spans.push((task_id, span));
                }
            }
            Ok(Event::Eof) | Err(_) => return spans,
            Ok(_) => {}
        }
    }
}

fn find_outgoing_flow_summaries(contents: &str, gateway_id: &str) -> Vec<OutgoingFlowSummary> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut active_flow: Option<(usize, String, bool)> = None;
    let mut flows = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "sequenceFlow") {
                    let source_ref = attribute_value(&reader, &event, "sourceRef");
                    let flow_id = attribute_value(&reader, &event, "id");
                    if source_ref.as_deref() == Some(gateway_id) {
                        active_flow = Some((depth, flow_id.unwrap_or_default(), false));
                    }
                } else if active_flow.is_some()
                    && is_element(&event, "conditionExpression")
                    && let Some((_flow_depth, _flow_id, has_condition)) = active_flow.as_mut()
                {
                    *has_condition = true;
                }
            }
            Ok(Event::Empty(event)) => {
                if is_element(&event, "sequenceFlow") {
                    let source_ref = attribute_value(&reader, &event, "sourceRef");
                    let flow_id = attribute_value(&reader, &event, "id");
                    if source_ref.as_deref() == Some(gateway_id)
                        && let Some(flow_id) = flow_id
                    {
                        flows.push(OutgoingFlowSummary {
                            id: flow_id,
                            has_condition: false,
                        });
                    }
                } else if active_flow.is_some()
                    && is_element(&event, "conditionExpression")
                    && let Some((_flow_depth, _flow_id, has_condition)) = active_flow.as_mut()
                {
                    *has_condition = true;
                }
            }
            Ok(Event::End(event)) => {
                if local_name(event.name().as_ref()) == "sequenceFlow"
                    && let Some((flow_depth, flow_id, has_condition)) = active_flow.take()
                {
                    if !flow_id.is_empty() {
                        flows.push(OutgoingFlowSummary {
                            id: flow_id,
                            has_condition,
                        });
                    }
                    if flow_depth != depth {
                        active_flow = None;
                    }
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return flows,
            Ok(_) => {}
        }
    }
}

fn start_event_span(event_end: usize, event: &BytesStart<'_>) -> Option<std::ops::Range<usize>> {
    let raw: &[u8] = event.as_ref();
    let start = event_end.checked_sub(raw.len() + 2)?;
    Some(start..event_end)
}

fn find_gateway_condition_expression_span(
    contents: &str,
    gateway_id: &str,
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
                return event_text_span(reader_position(&reader)?, event.as_ref());
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                return event_text_span(reader_position(&reader)?, event.as_ref());
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

fn find_gateway_condition_expression_text(contents: &str, gateway_id: &str) -> Option<String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut sequence_flow_depth = None;
    let mut depth = 0usize;
    let mut in_condition_expression = false;
    let mut condition_text = String::new();

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
                    condition_text.clear();
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
                condition_text.push_str(event.decode().ok()?.as_ref());
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok()?.as_ref());
            }
            Ok(Event::GeneralRef(event)) if in_condition_expression => {
                let reference = event.decode().ok()?;
                if let Some(entity) = resolve_predefined_entity(reference.as_ref()) {
                    condition_text.push_str(entity);
                } else {
                    condition_text.push('&');
                    condition_text.push_str(reference.as_ref());
                    condition_text.push(';');
                }
            }
            Ok(Event::End(event)) => {
                if local_name(event.name().as_ref()) == "conditionExpression" {
                    if !condition_text.trim().is_empty() {
                        return Some(condition_text.trim().to_string());
                    }
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

fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
}

fn event_text_span(event_end: usize, raw_text: &[u8]) -> Option<std::ops::Range<usize>> {
    let event_start = event_end.checked_sub(raw_text.len())?;
    let leading = raw_text
        .iter()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count();
    let trailing = raw_text
        .iter()
        .rev()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count();
    Some((event_start + leading)..(event_end - trailing))
}

fn unsupported_condition_expression_help(condition: &str) -> String {
    let decoded = decode_xml_text(condition);
    if let Some((lhs, operator, rhs)) = variable_to_variable_comparison(&decoded) {
        return format!(
            "Unsupported variable-to-variable comparison `{lhs} {operator} {rhs}`. Emit one boolean such as `hasMoreSections` from the upstream task and route on that boolean, or emit one numeric count such as `sectionsRemaining` and compare it to a numeric literal like `sectionsRemaining > 0`. Do not compare two variables directly in the gateway condition."
        );
    }
    "Use one boolean path such as `approved` or `not approved`, or one numeric comparison from a variable to a numeric literal such as `amount > 100`. For variable-to-variable decisions, emit an upstream boolean route variable and branch on that. Return a minimal unified diff only.".to_string()
}

fn variable_to_variable_comparison(condition: &str) -> Option<(&str, &str, &str)> {
    for operator in ["<=", ">=", "==", "!=", ">", "<"] {
        let Some(index) = condition.find(operator) else {
            continue;
        };
        let lhs = condition[..index].trim();
        let rhs = condition[index + operator.len()..].trim();
        if is_variable_operand_hint(lhs) && is_variable_operand_hint(rhs) {
            return Some((lhs, operator, rhs));
        }
    }
    None
}

fn is_variable_operand_hint(source: &str) -> bool {
    !matches!(source, "true" | "false" | "null")
        && source.parse::<f64>().is_err()
        && is_identifier_path_hint(source)
}

fn is_identifier_path_hint(path: &str) -> bool {
    !path.is_empty() && path.split('.').all(is_identifier_segment_hint)
}

fn is_identifier_segment_hint(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn decode_xml_text(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn is_element(event: &BytesStart<'_>, expected: &str) -> bool {
    local_name(event.name().as_ref()) == expected
}

fn is_routable_task_element(event: &BytesStart<'_>) -> bool {
    matches!(
        local_name(event.name().as_ref()),
        "serviceTask"
            | "scriptTask"
            | "userTask"
            | "manualTask"
            | "businessRuleTask"
            | "sendTask"
            | "receiveTask"
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

fn local_name(raw: &[u8]) -> &str {
    std::str::from_utf8(raw)
        .ok()
        .map_or("", |name| name.rsplit(':').next().unwrap_or(name))
}
