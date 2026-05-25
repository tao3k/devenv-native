//! Source-level BPMN lint diagnostics.

mod core;
mod xml;

pub(super) use core::{
    ActiveGatewayFlow, GatewayFlowDetail, InvalidDefaultFlowContext, MissingBranchConditionContext,
    OutgoingFlowSummary, append_unique_source_issues, find_default_branching_context,
    find_invalid_default_flow_context, find_task_routing_violations, preferred_default_flow,
    should_append_source_gateway_condition_issues, should_append_source_task_routing_issue,
    should_append_source_unsupported_condition_issues,
    source_duplicate_unconditional_gateway_issues, source_invalid_default_gateway_issues,
    source_issue_group_size, source_task_routing_issue, task_routing_structured_repair,
    task_routing_violations_json, task_routing_violations_summary,
};
pub(super) use xml::{
    escaped_line_fix_for_ampersand, find_gateway_condition_expression_span,
    find_gateway_condition_expression_text, find_missing_branch_condition_context,
    find_unescaped_ampersand_span, find_unescaped_placeholder_span, find_xml_error_token_span,
    malformed_closing_tag_line_fix, unsupported_condition_expression_help,
};
