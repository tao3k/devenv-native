use super::evidence::{augment_evidence, decision_display, root_context};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn issue_from_dmn_hit_policy_error(
    error: &BpmnEngineError,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnsupportedDmnHitPolicy {
            source_id,
            decision_id,
            hit_policy,
        } => LintIssue::new(
            "dmn.unsupported_hit_policy",
            "DMN hit policy is outside the supported subset",
            format!(
                "{} in source '{source_id}' uses unsupported hit policy '{hit_policy}'.",
                decision_display(decision_id, snapshot)
            ),
            format!(
                "The bounded evaluator currently supports only `UNIQUE` and `COLLECT`.{}",
                root_context(snapshot)
            ),
            vec![
                "Change the decisionTable hitPolicy to `UNIQUE` or `COLLECT` if the rule semantics allow it.".to_string(),
                "If the original hit policy is required, preserve the original design notes and defer execution until broader support exists.".to_string(),
            ],
            format!(
                "Edit DMN source '{source_id}' so decision '{decision_id}' uses a supported hit policy. Prefer `UNIQUE` or `COLLECT`, and preserve original rule intent when converting from '{hit_policy}'."
            ),
            augment_evidence(json!({
                "source_id": source_id,
                "decision_id": decision_id,
                "hit_policy": hit_policy,
            }), snapshot, Some(decision_id)),
        ),
        _ => return None,
    })
}

pub(super) fn issue_from_dmn_expression_subset_error(
    error: &BpmnEngineError,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnsupportedDmnLiteral { source_id, literal } => LintIssue::new(
            "dmn.unsupported_literal",
            "DMN literal expression is outside the supported subset",
            format!("Source '{source_id}' uses unsupported literal expression '{literal}'."),
            format!(
                "The bounded evaluator only accepts wildcard `-` and literal equality for strings, numbers, booleans, `null`, ISO date literals like `date(\"2026-01-01\")`, signed ISO 8601 day-time duration literals like `duration(\"-PT30M\")`, `duration(\"PT30M\")`, `duration(\"P1DT2H\")`, `duration(\"P1.5D\")`, `duration(\"P1,5D\")`, `duration(\"PT1.5H\")`, `duration(\"PT1,5H\")`, `duration(\"PT1.5M\")`, `duration(\"PT1,5M\")`, `duration(\"PT1.5S\")`, or `duration(\"PT1,5S\")`, signed ISO 8601 year-month duration literals like `duration(\"-P6M\")`, `duration(\"P6M\")`, or `duration(\"P1Y\")`, ISO datetime literals like `date and time(\"2026-01-01T09:00:00\")` or RFC3339 offset-aware forms like `date and time(\"2026-01-01T09:00:00Z\")`, and ISO time literals like `time(\"09:00:00\")`.{}",
                root_context(snapshot)
            ),
            vec![
                "Replace the expression with a supported literal form or wildcard `-`.".to_string(),
                "Use `date(\"YYYY-MM-DD\")` for date-only literals, `duration(\"-PT30M\")`, `duration(\"PT30M\")`, `duration(\"P1DT2H\")`, `duration(\"P1.5D\")`, `duration(\"P1,5D\")`, `duration(\"PT1.5H\")`, `duration(\"PT1,5H\")`, `duration(\"PT1.5M\")`, `duration(\"PT1,5M\")`, `duration(\"PT1.5S\")`, or `duration(\"PT1,5S\")` for signed day-time durations, `duration(\"-P6M\")`, `duration(\"P6M\")`, or `duration(\"P1Y\")` for signed year-month durations, `date and time(\"YYYY-MM-DDTHH:MM:SS\")` or `date and time(\"YYYY-MM-DDTHH:MM:SSZ\")` for datetime literals, and `time(\"HH:MM:SS\")` for time-only literals; keep trailing-lower-unit forms such as `duration(\"PT1.5H30S\")`, mixed year-month/day-time duration forms, fractional year-month forms, and custom functions deferred.".to_string(),
            ],
            format!(
                "Edit DMN source '{source_id}' so literal expression '{literal}' is replaced with a supported bounded form: wildcard `-`, quoted string, number, boolean, `null`, ISO date literal `date(\"YYYY-MM-DD\")`, signed day-time duration literal `duration(\"-PT30M\")`, `duration(\"PT30M\")`, `duration(\"P1DT2H\")`, `duration(\"P1.5D\")`, `duration(\"P1,5D\")`, `duration(\"PT1.5H\")`, `duration(\"PT1,5H\")`, `duration(\"PT1.5M\")`, `duration(\"PT1,5M\")`, `duration(\"PT1.5S\")`, or `duration(\"PT1,5S\")`, signed year-month duration literal `duration(\"-P6M\")`, `duration(\"P6M\")`, or `duration(\"P1Y\")`, ISO datetime literal `date and time(\"YYYY-MM-DDTHH:MM:SS\")` or `date and time(\"YYYY-MM-DDTHH:MM:SSZ\")`, or ISO time literal `time(\"HH:MM:SS\")`."
            ),
            augment_evidence(json!({
                "source_id": source_id,
                "literal": literal,
            }), snapshot, None),
        ),
        BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id,
            expression,
        } => LintIssue::new(
            "dmn.unsupported_unary_test",
            "DMN unary test is outside the supported subset",
            format!("Source '{source_id}' uses unsupported unary test '{expression}'."),
            format!(
                "The bounded evaluator accepts wildcard `-`, literal equality including `date(\"YYYY-MM-DD\")`, signed day-time duration literals like `duration(\"-PT30M\")`, `duration(\"PT30M\")`, `duration(\"P1DT2H\")`, `duration(\"P1.5D\")`, `duration(\"P1,5D\")`, `duration(\"PT1.5H\")`, `duration(\"PT1,5H\")`, `duration(\"PT1.5M\")`, `duration(\"PT1,5M\")`, `duration(\"PT1.5S\")`, or `duration(\"PT1,5S\")`, signed year-month duration literals like `duration(\"-P6M\")`, `duration(\"P6M\")`, or `duration(\"P1Y\")`, `date and time(\"YYYY-MM-DDTHH:MM:SS\")`, RFC3339 offset-aware datetime forms like `date and time(\"YYYY-MM-DDTHH:MM:SSZ\")`, and `time(\"HH:MM:SS\")`, numeric comparisons like `< 25` or `>= 25`, bounded numeric ranges like `100 <= ? <= 110` or `[100..110]`, signed day-time duration comparisons like `< duration(\"PT1H\")`, `< duration(\"-PT30M\")`, `< duration(\"PT2.25H\")`, `< duration(\"PT2,25H\")`, `< duration(\"PT2.25S\")`, or `< duration(\"PT2,25S\")`, signed year-month duration comparisons like `< duration(\"P1Y\")` or `< duration(\"-P6M\")`, bounded duration ranges like `duration(\"PT15M\") <= ? < duration(\"PT45M\")`, `duration(\"PT1.5M\") <= ? < duration(\"PT2.75M\")`, `duration(\"PT1,5M\") <= ? < duration(\"PT2,75M\")`, `[duration(\"P1DT1H\")..duration(\"P1DT2H\")]`, `[duration(\"P1.25D\")..duration(\"P1.5D\")]`, `[duration(\"P1,25D\")..duration(\"P1,5D\")]`, `duration(\"-PT30M\") <= ? < duration(\"PT0S\")`, `duration(\"-PT0.5S\") <= ? < duration(\"PT0.5S\")`, `duration(\"-P6M\") <= ? < duration(\"P0M\")`, `duration(\"P6M\") <= ? < duration(\"P1Y\")`, or `[duration(\"P1Y\")..duration(\"P2Y\")]`, ISO date comparisons like `< date(\"2026-01-01\")`, bounded ISO date ranges like `date(\"2026-01-01\") <= ? < date(\"2026-01-31\")` or `[date(\"2026-01-01\")..date(\"2026-01-31\")]`, ISO datetime comparisons like `< date and time(\"2026-01-01T09:00:00\")` or `< date and time(\"2026-01-01T09:00:00Z\")`, bounded ISO datetime ranges like `date and time(\"2026-01-01T09:00:00\") <= ? < date and time(\"2026-01-01T17:00:00\")` or `date and time(\"2026-01-01T09:00:00Z\") <= ? < date and time(\"2026-01-01T17:00:00Z\")`, and ISO time comparisons like `< time(\"09:00:00\")`, and bounded ISO time ranges like `time(\"09:00:00\") <= ? < time(\"17:00:00\")` or `[time(\"09:00:00\")..time(\"17:00:00\")]`.{}",
                root_context(snapshot)
            ),
            vec![
                "Prefer literal equality when a single exact value is enough.".to_string(),
                "For numeric thresholds, use one comparison operator with one numeric bound.".to_string(),
                "For numeric intervals, use one bounded range such as `100 <= ? <= 110` or `[100..110]`.".to_string(),
                "For signed day-time duration thresholds, use `duration(\"-PT30M\")`, `duration(\"PT30M\")`, `duration(\"P1DT2H\")`, bounded fractional day-time forms like `duration(\"P1.5D\")`, `duration(\"P1,5D\")`, `duration(\"PT1.5H\")`, `duration(\"PT1,5H\")`, `duration(\"PT1.5M\")`, `duration(\"PT1,5M\")`, `duration(\"PT1.5S\")`, or `duration(\"PT1,5S\")`, one comparison operator, or one bounded range.".to_string(),
                "For signed year-month duration thresholds, use `duration(\"-P6M\")`, `duration(\"P6M\")`, `duration(\"P1Y\")`, one comparison operator, or one bounded range.".to_string(),
                "For date-only thresholds, use `date(\"YYYY-MM-DD\")` with one comparison operator or one bounded range.".to_string(),
                "For datetime thresholds, use `date and time(\"YYYY-MM-DDTHH:MM:SS\")` or `date and time(\"YYYY-MM-DDTHH:MM:SSZ\")` with one comparison operator or one bounded range.".to_string(),
                "For time-only thresholds, use `time(\"HH:MM:SS\")` with one comparison operator or one bounded range.".to_string(),
                "Keep trailing-lower-unit forms such as `duration(\"PT1.5H30S\")`, mixed year-month/day-time duration forms, fractional year-month forms, custom functions, and broader FEEL expressions deferred in this slice.".to_string(),
            ],
            format!(
                "Edit DMN source '{source_id}' so unary test '{expression}' uses one supported bounded form: wildcard `-`, literal equality including `date(\"YYYY-MM-DD\")`, signed day-time duration literals like `duration(\"-PT30M\")`, `duration(\"PT30M\")`, `duration(\"P1DT2H\")`, `duration(\"P1.5D\")`, `duration(\"P1,5D\")`, `duration(\"PT1.5H\")`, `duration(\"PT1,5H\")`, `duration(\"PT1.5M\")`, `duration(\"PT1,5M\")`, `duration(\"PT1.5S\")`, or `duration(\"PT1,5S\")`, signed year-month duration literals like `duration(\"-P6M\")`, `duration(\"P6M\")`, or `duration(\"P1Y\")`, `date and time(\"YYYY-MM-DDTHH:MM:SS\")`, `date and time(\"YYYY-MM-DDTHH:MM:SSZ\")`, or `time(\"HH:MM:SS\")`, one numeric comparison (`<`, `<=`, `>`, `>=`), one bounded numeric range like `100 <= ? <= 110` or `[100..110]`, one signed day-time duration comparison like `< duration(\"PT1H\")`, `< duration(\"-PT30M\")`, `< duration(\"PT2.25H\")`, `< duration(\"PT2,25H\")`, `< duration(\"PT2.25S\")`, or `< duration(\"PT2,25S\")`, one signed year-month duration comparison like `< duration(\"P1Y\")` or `< duration(\"-P6M\")`, one bounded duration range like `duration(\"PT15M\") <= ? < duration(\"PT45M\")`, `duration(\"PT1.5M\") <= ? < duration(\"PT2.75M\")`, `duration(\"PT1,5M\") <= ? < duration(\"PT2,75M\")`, `[duration(\"P1DT1H\")..duration(\"P1DT2H\")]`, `[duration(\"P1.25D\")..duration(\"P1.5D\")]`, `[duration(\"P1,25D\")..duration(\"P1,5D\")]`, `duration(\"-PT30M\") <= ? < duration(\"PT0S\")`, `duration(\"-PT0.5S\") <= ? < duration(\"PT0.5S\")`, `duration(\"-P6M\") <= ? < duration(\"P0M\")`, `duration(\"P6M\") <= ? < duration(\"P1Y\")`, or `[duration(\"P1Y\")..duration(\"P2Y\")]`, one ISO date comparison like `< date(\"2026-01-01\")`, one bounded ISO date range like `date(\"2026-01-01\") <= ? < date(\"2026-01-31\")` or `[date(\"2026-01-01\")..date(\"2026-01-31\")]`, one ISO datetime comparison like `< date and time(\"2026-01-01T09:00:00\")` or `< date and time(\"2026-01-01T09:00:00Z\")`, one bounded ISO datetime range like `date and time(\"2026-01-01T09:00:00\") <= ? < date and time(\"2026-01-01T17:00:00\")`, `date and time(\"2026-01-01T09:00:00Z\") <= ? < date and time(\"2026-01-01T17:00:00Z\")`, or the corresponding interval form, one ISO time comparison like `< time(\"09:00:00\")`, or one bounded ISO time range like `time(\"09:00:00\") <= ? < time(\"17:00:00\")` or `[time(\"09:00:00\")..time(\"17:00:00\")]`."
            ),
            augment_evidence(json!({
                "source_id": source_id,
                "expression": expression,
            }), snapshot, None),
        ),
        _ => return None,
    })
}

pub(super) fn issue_from_dmn_table_error(
    error: &BpmnEngineError,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::InvalidDmnRuleArity {
            source_id,
            rule_id,
            expected_inputs,
            actual_inputs,
            expected_outputs,
            actual_outputs,
        } => LintIssue::new(
            "dmn.invalid_rule_arity",
            "DMN rule entry count does not match the table clauses",
            format!(
                "Rule '{rule_id}' in source '{source_id}' has {actual_inputs}/{actual_outputs} input/output entries, but the table expects {expected_inputs}/{expected_outputs}."
            ),
            format!(
                "Every rule must provide one input entry per input clause and one output entry per output clause.{}",
                root_context(snapshot)
            ),
            vec![
                "Count the table input and output clauses first.".to_string(),
                "Add or remove rule entries so the rule arity exactly matches the table clause counts.".to_string(),
            ],
            format!(
                "Repair DMN source '{source_id}' so rule '{rule_id}' has exactly {expected_inputs} input entries and {expected_outputs} output entries, matching the decision table clauses."
            ),
            augment_evidence(json!({
                "source_id": source_id,
                "rule_id": rule_id,
                "expected_inputs": expected_inputs,
                "actual_inputs": actual_inputs,
                "expected_outputs": expected_outputs,
                "actual_outputs": actual_outputs,
            }), snapshot, None),
        ),
        _ => return None,
    })
}
