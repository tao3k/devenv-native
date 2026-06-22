use super::{GatewayConditionSummary, HashSet, parse_gateway_condition_summary};

pub(super) fn is_task_tag(tag: &str) -> bool {
    matches!(
        tag,
        "task"
            | "sendTask"
            | "serviceTask"
            | "userTask"
            | "manualTask"
            | "businessRuleTask"
            | "scriptTask"
    )
}

pub(super) fn gateway_condition_variable_path(condition: &str) -> Option<String> {
    match parse_gateway_condition_summary(condition)? {
        GatewayConditionSummary::BooleanPath { path, .. } => Some(path),
        GatewayConditionSummary::NumericComparison { lhs, .. } => Some(lhs),
    }
}

pub(super) fn declares_gateway_variable(outputs: &HashSet<String>, variable_path: &str) -> bool {
    let root = variable_path.split('.').next().unwrap_or(variable_path);
    outputs.contains(variable_path) || outputs.contains(root)
}
