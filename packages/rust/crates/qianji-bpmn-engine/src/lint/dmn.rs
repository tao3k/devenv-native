//! DMN lint entrypoint and error-to-guidance mapping.

use super::{LintDomain, LintIssue, LintReport};
use crate::dmn::{DmnSourceFile, parse_dmn_decision};
use crate::error::BpmnEngineError;
use serde_json::json;

/// Lints one DMN source and returns an LLM-friendly blocking report.
#[must_use]
pub fn lint_dmn_source(source: &DmnSourceFile) -> LintReport {
    match parse_dmn_decision(source) {
        Ok(_) => LintReport::ok(LintDomain::Dmn, &source.source_id),
        Err(error) => LintReport::blocking(
            LintDomain::Dmn,
            &source.source_id,
            vec![issue_from_dmn_error(source, &error)],
        ),
    }
}

fn issue_from_dmn_error(source: &DmnSourceFile, error: &BpmnEngineError) -> LintIssue {
    issue_from_dmn_document_error(error)
        .or_else(|| issue_from_dmn_contract_error(error))
        .or_else(|| issue_from_dmn_table_error(error))
        .unwrap_or_else(|| unexpected_dmn_issue(source, error))
}

fn issue_from_dmn_document_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::InvalidDmnXml { source_id, message } => LintIssue::new(
            "dmn.invalid_xml",
            "DMN XML is not well-formed",
            format!("Source '{source_id}' cannot be parsed as DMN XML: {message}"),
            "The DMN parser stops before decision-table validation when the XML tree is malformed.",
            vec![
                "Repair the XML structure first: close tags, fix attributes, and restore valid nesting.".to_string(),
                "Preserve decision ids, table ids, and rule ids while repairing XML syntax.".to_string(),
            ],
            format!(
                "Repair the XML syntax in DMN source '{source_id}' so it becomes well-formed without changing decision semantics. Preserve ids, hit policies, and rule ordering while fixing XML structure."
            ),
            json!({
                "source_id": source_id,
                "parser_message": message,
            }),
        ),
        BpmnEngineError::MissingDmnRootElement { source_id } => LintIssue::new(
            "dmn.missing_root_element",
            "DMN file has no root XML element",
            format!("Source '{source_id}' does not contain a root DMN XML element."),
            "The linter cannot discover `<definitions>` or any decision content when the file is empty or structurally missing its root node.",
            vec![
                "Add one DMN XML root element, typically `<definitions>`, around the decision content.".to_string(),
                "Move decisions and decision tables inside that root element.".to_string(),
            ],
            format!(
                "Rewrite DMN source '{source_id}' so it has one valid root element, typically `<definitions>`, and place all decision content inside it."
            ),
            json!({
                "source_id": source_id,
            }),
        ),
        BpmnEngineError::MissingDmnAttribute {
            source_id,
            element,
            attribute,
        } => LintIssue::new(
            "dmn.missing_attribute",
            "Required DMN attribute is missing",
            format!(
                "Element '<{element}>' in source '{source_id}' is missing required attribute '{attribute}'."
            ),
            "The bounded DMN parser needs this attribute to identify a decision, table, clause, or rule consistently.",
            vec![
                format!("Add the missing '{attribute}' attribute on `<{element}>`."),
                "Use a stable identifier or value that remains consistent with the surrounding decision-table structure.".to_string(),
            ],
            format!(
                "Edit DMN source '{source_id}' and add the required '{attribute}' attribute to `<{element}>`. Keep related ids and references stable so the decision table remains coherent."
            ),
            json!({
                "source_id": source_id,
                "element": element,
                "attribute": attribute,
            }),
        ),
        BpmnEngineError::MissingDmnDecision { source_id } => LintIssue::new(
            "dmn.missing_decision",
            "DMN file contains no decisions",
            format!("Source '{source_id}' does not contain any `<decision>` elements."),
            "The bounded DMN contract expects one decision per linted source.",
            vec![
                "Add one `<decision>` element to the file.".to_string(),
                "Nest exactly one `<decisionTable>` under that decision for the bounded v1 contract.".to_string(),
            ],
            format!(
                "Rewrite DMN source '{source_id}' so it contains exactly one `<decision>` element with one nested `<decisionTable>`."
            ),
            json!({
                "source_id": source_id,
            }),
        ),
        _ => return None,
    })
}

fn issue_from_dmn_contract_error(error: &BpmnEngineError) -> Option<LintIssue> {
    issue_from_dmn_table_shape_error(error)
        .or_else(|| issue_from_dmn_hit_policy_error(error))
        .or_else(|| issue_from_dmn_expression_subset_error(error))
}

fn issue_from_dmn_table_shape_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnsupportedDmnDecisionCount { source_id, count } => LintIssue::new(
            "dmn.unsupported_decision_count",
            "DMN source has too many decisions for the bounded contract",
            format!(
                "Source '{source_id}' contains {count} decisions, but the bounded linter expects exactly 1."
            ),
            "The current DMN slice only supports one decision per source so the later adapter can bind one business-rule target deterministically.",
            vec![
                "Split multiple decisions into separate DMN files when possible.".to_string(),
                "If the decisions must stay related, keep one primary decision in this file and defer the rest until multi-decision support exists.".to_string(),
            ],
            format!(
                "Edit DMN source '{source_id}' so it contains exactly one `<decision>` for the bounded engine contract. Split extra decisions into separate files rather than merging them into one table."
            ),
            json!({
                "source_id": source_id,
                "decision_count": count,
            }),
        ),
        BpmnEngineError::MissingDmnDecisionTable { decision_id } => LintIssue::new(
            "dmn.missing_decision_table",
            "DMN decision has no decision table",
            format!("Decision '{decision_id}' does not contain a `<decisionTable>`."),
            "The bounded evaluator only understands decision-table backed decisions in this slice.",
            vec![
                "Add exactly one `<decisionTable>` under the decision.".to_string(),
                "Move inputs, outputs, and rules inside that table rather than leaving them at the decision level.".to_string(),
            ],
            format!(
                "Repair decision '{decision_id}' by adding exactly one `<decisionTable>` and placing all input, output, and rule clauses inside it."
            ),
            json!({
                "decision_id": decision_id,
            }),
        ),
        BpmnEngineError::UnsupportedDmnDecisionTableCount { decision_id, count } => {
            LintIssue::new(
                "dmn.unsupported_decision_table_count",
                "DMN decision has too many decision tables",
                format!(
                    "Decision '{decision_id}' contains {count} decision tables, but the bounded contract expects exactly 1."
                ),
                "The current evaluator resolves one decision to one table so adapter wiring stays deterministic.",
                vec![
                    "Keep one decision table in the file and move extra tables into separate decisions or files.".to_string(),
                    "Do not merge unrelated tables if that would change rule meaning.".to_string(),
                ],
                format!(
                    "Edit decision '{decision_id}' so it contains exactly one `<decisionTable>`. Split extra tables into separate decisions or files instead of forcing a lossy merge."
                ),
                json!({
                    "decision_id": decision_id,
                    "table_count": count,
                }),
            )
        }
        _ => return None,
    })
}

fn issue_from_dmn_hit_policy_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnsupportedDmnHitPolicy {
            source_id,
            decision_id,
            hit_policy,
        } => LintIssue::new(
            "dmn.unsupported_hit_policy",
            "DMN hit policy is outside the supported subset",
            format!(
                "Decision '{decision_id}' in source '{source_id}' uses unsupported hit policy '{hit_policy}'."
            ),
            "The bounded evaluator currently supports only `UNIQUE` and `COLLECT`.",
            vec![
                "Change the decisionTable hitPolicy to `UNIQUE` or `COLLECT` if the rule semantics allow it.".to_string(),
                "If the original hit policy is required, preserve the original design notes and defer execution until broader support exists.".to_string(),
            ],
            format!(
                "Edit DMN source '{source_id}' so decision '{decision_id}' uses a supported hit policy. Prefer `UNIQUE` or `COLLECT`, and preserve original rule intent when converting from '{hit_policy}'."
            ),
            json!({
                "source_id": source_id,
                "decision_id": decision_id,
                "hit_policy": hit_policy,
            }),
        ),
        _ => return None,
    })
}

fn issue_from_dmn_expression_subset_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnsupportedDmnLiteral { source_id, literal } => LintIssue::new(
            "dmn.unsupported_literal",
            "DMN literal expression is outside the supported subset",
            format!("Source '{source_id}' uses unsupported literal expression '{literal}'."),
            "The bounded evaluator only accepts wildcard `-` and literal equality for strings, numbers, booleans, `null`, and ISO date literals like `date(\"2026-01-01\")`.",
            vec![
                "Replace the expression with a supported literal form or wildcard `-`.".to_string(),
                "Use `date(\"YYYY-MM-DD\")` only for date-only literals; keep `time(...)`, `date and time(...)`, durations, and custom functions deferred.".to_string(),
            ],
            format!(
                "Edit DMN source '{source_id}' so literal expression '{literal}' is replaced with a supported bounded form: wildcard `-`, quoted string, number, boolean, `null`, or ISO date literal `date(\"YYYY-MM-DD\")`."
            ),
            json!({
                "source_id": source_id,
                "literal": literal,
            }),
        ),
        BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id,
            expression,
        } => LintIssue::new(
            "dmn.unsupported_unary_test",
            "DMN unary test is outside the supported subset",
            format!("Source '{source_id}' uses unsupported unary test '{expression}'."),
            "The bounded evaluator accepts wildcard `-`, literal equality including `date(\"YYYY-MM-DD\")`, numeric comparisons like `< 25` or `>= 25`, bounded numeric ranges like `100 <= ? <= 110` or `[100..110]`, ISO date comparisons like `< date(\"2026-01-01\")`, and bounded ISO date ranges like `date(\"2026-01-01\") <= ? < date(\"2026-01-31\")` or `[date(\"2026-01-01\")..date(\"2026-01-31\")]`.",
            vec![
                "Prefer literal equality when a single exact value is enough.".to_string(),
                "For numeric thresholds, use one comparison operator with one numeric bound.".to_string(),
                "For numeric intervals, use one bounded range such as `100 <= ? <= 110` or `[100..110]`.".to_string(),
                "For date-only thresholds, use `date(\"YYYY-MM-DD\")` with one comparison operator or one bounded range.".to_string(),
                "Keep `time(...)`, `date and time(...)`, durations, functions, and broader FEEL expressions deferred in this slice.".to_string(),
            ],
            format!(
                "Edit DMN source '{source_id}' so unary test '{expression}' uses one supported bounded form: wildcard `-`, literal equality including `date(\"YYYY-MM-DD\")`, one numeric comparison (`<`, `<=`, `>`, `>=`), one bounded numeric range like `100 <= ? <= 110` or `[100..110]`, one ISO date comparison like `< date(\"2026-01-01\")`, or one bounded ISO date range like `date(\"2026-01-01\") <= ? < date(\"2026-01-31\")` or `[date(\"2026-01-01\")..date(\"2026-01-31\")]`."
            ),
            json!({
                "source_id": source_id,
                "expression": expression,
            }),
        ),
        _ => return None,
    })
}

fn issue_from_dmn_table_error(error: &BpmnEngineError) -> Option<LintIssue> {
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
            "Every rule must provide one input entry per input clause and one output entry per output clause.",
            vec![
                "Count the table input and output clauses first.".to_string(),
                "Add or remove rule entries so the rule arity exactly matches the table clause counts.".to_string(),
            ],
            format!(
                "Repair DMN source '{source_id}' so rule '{rule_id}' has exactly {expected_inputs} input entries and {expected_outputs} output entries, matching the decision table clauses."
            ),
            json!({
                "source_id": source_id,
                "rule_id": rule_id,
                "expected_inputs": expected_inputs,
                "actual_inputs": actual_inputs,
                "expected_outputs": expected_outputs,
                "actual_outputs": actual_outputs,
            }),
        ),
        _ => return None,
    })
}

fn unexpected_dmn_issue(source: &DmnSourceFile, error: &BpmnEngineError) -> LintIssue {
    LintIssue::new(
        "dmn.unexpected_engine_error",
        "Unexpected DMN lint error",
        format!(
            "DMN linting for source '{}' returned unexpected engine error: {error}",
            source.source_id
        ),
        "The linter expected a DMN parse or validation error but received a broader engine error, which usually indicates a missing lint mapping.",
        vec![
            "Inspect the emitted evidence before rewriting the DMN source.".to_string(),
            "If the DMN appears valid, extend the linter mapping rather than forcing an unsafe rewrite.".to_string(),
        ],
        format!(
            "Do not rewrite DMN source '{}' blindly. First inspect the unexpected engine error and repair only the concrete problem proven by the evidence.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "engine_error": error.to_string(),
        }),
    )
}
