use super::{
    ActiveGatewayFlow, BpmnSourceFile, GatewayConditionSummary, LintIssue,
    UnsupportedGatewayCondition, UnsupportedGatewayConditionGroup,
    ambiguous_boolean_condition_issue, ambiguous_boolean_path_kind, is_supported_gateway_condition,
    parse_gateway_condition_summary, unsupported_gateway_condition_issue,
};

pub(super) fn source_ambiguous_boolean_condition_issue(
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

pub(super) fn source_unsupported_gateway_condition(
    active_flow: Option<&ActiveGatewayFlow>,
    condition: Option<&str>,
) -> Option<UnsupportedGatewayCondition> {
    let flow = active_flow?;
    let condition = condition?.trim();
    if condition.is_empty() || is_supported_gateway_condition(condition) {
        return None;
    }
    Some(UnsupportedGatewayCondition {
        process_id: (flow.process_id.clone()),
        gateway_id: flow.gateway_id.clone(),
        condition: condition.to_string(),
    })
}

pub(super) fn grouped_unsupported_gateway_condition_issues(
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
                process_id: (condition.process_id),
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
