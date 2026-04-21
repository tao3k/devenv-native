use super::evidence::{augment_evidence, decision_display, root_context};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn unsupported_context_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_context_decision",
        "DMN decision uses context logic instead of a decision table",
        format!(
            "{} uses direct `<context>` logic and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator only executes decision-table backed decisions in this slice; direct context decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, name, and context-entry names while deciding whether this logic can be expressed as one bounded `<decisionTable>`.".to_string(),
            "Do not flatten context entries into guessed rules unless the mapping from each context entry to bounded decision-table clauses is explicit and lossless.".to_string(),
            "If no safe table conversion exists yet, keep the source as a non-executable DMN placeholder and report unsupported `context` execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not rewrite its direct `<context>` into guessed decision-table rules. Only replace the context with one equivalent bounded `<decisionTable>` when the entry-to-rule mapping is explicit and lossless; otherwise keep the decision non-executable and report unsupported context execution."
        ),
        augment_evidence(json!({
            "decision_id": decision_id,
        }), snapshot, Some(decision_id)),
    )
}

pub(super) fn unsupported_invocation_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_invocation_decision",
        "DMN decision uses invocation logic instead of a decision table",
        format!(
            "{} uses direct `<invocation>` logic and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator only executes decision-table backed decisions in this slice; direct invocation decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, name, and invocation bindings while deciding whether this logic can be expressed as one bounded `<decisionTable>`.".to_string(),
            "Do not inline or fabricate invoked logic unless the called function semantics and every binding-to-rule mapping are explicit and lossless.".to_string(),
            "If no safe table conversion exists yet, keep the source as a non-executable DMN placeholder and report unsupported `invocation` execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not rewrite its direct `<invocation>` into guessed decision-table rules. Only replace the invocation with one equivalent bounded `<decisionTable>` when the function and binding semantics map explicitly and losslessly to bounded rules; otherwise keep the decision non-executable and report unsupported invocation execution."
        ),
        augment_evidence(json!({
            "decision_id": decision_id,
        }), snapshot, Some(decision_id)),
    )
}

pub(super) fn unsupported_literal_expression_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_literal_expression_decision",
        "DMN decision uses literal expression logic instead of a decision table",
        format!(
            "{} uses direct `<literalExpression>` logic and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator only executes decision-table backed decisions in this slice; direct literal-expression decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id and name while deciding whether this logic can be expressed as one bounded `<decisionTable>`.".to_string(),
            "Do not silently approximate the FEEL expression; convert it only if the equivalent decision-table rules are explicit and lossless.".to_string(),
            "If no safe table conversion exists yet, keep the source as a non-executable DMN placeholder and report unsupported `literalExpression` execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not rewrite its `<literalExpression>` into guessed rules. Only replace the direct literal expression with one equivalent bounded `<decisionTable>` when the rule mapping is explicit and lossless; otherwise keep the decision non-executable and report unsupported literal-expression execution."
        ),
        augment_evidence(json!({
            "decision_id": decision_id,
        }), snapshot, Some(decision_id)),
    )
}

pub(super) fn unsupported_relation_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_relation_decision",
        "DMN decision uses relation logic instead of a decision table",
        format!(
            "{} uses direct `<relation>` logic and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator only executes decision-table backed decisions in this slice; direct relation decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, relation columns, and row ordering while deciding whether this logic can be expressed as one bounded `<decisionTable>`.".to_string(),
            "Do not flatten relation rows into guessed decision-table rules unless the column-to-clause mapping and every row-to-rule mapping are explicit and lossless.".to_string(),
            "If no safe table conversion exists yet, keep the source as a non-executable DMN placeholder and report unsupported `relation` execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not rewrite its direct `<relation>` into guessed decision-table rules. Only replace the relation with one equivalent bounded `<decisionTable>` when the columns, rows, and rule mapping are explicit and lossless; otherwise keep the decision non-executable and report unsupported relation execution."
        ),
        augment_evidence(json!({
            "decision_id": decision_id,
        }), snapshot, Some(decision_id)),
    )
}

pub(super) fn unsupported_function_definition_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_function_definition_decision",
        "DMN decision uses function definition logic instead of a decision table",
        format!(
            "{} uses direct `<functionDefinition>` logic and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator only executes decision-table backed decisions in this slice; direct functionDefinition decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, formal parameters, and function body while deciding whether this logic can be expressed as one bounded `<decisionTable>`.".to_string(),
            "Do not inline or approximate function semantics unless the parameter-to-clause mapping and body-to-rule mapping are explicit and lossless.".to_string(),
            "If no safe table conversion exists yet, keep the source as a non-executable DMN placeholder and report unsupported `functionDefinition` execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not rewrite its direct `<functionDefinition>` into guessed decision-table rules. Only replace the function definition with one equivalent bounded `<decisionTable>` when the parameters, body, and rule mapping are explicit and lossless; otherwise keep the decision non-executable and report unsupported function-definition execution."
        ),
        augment_evidence(json!({
            "decision_id": decision_id,
        }), snapshot, Some(decision_id)),
    )
}

pub(super) fn unsupported_list_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_list_decision",
        "DMN decision uses list logic instead of a decision table",
        format!(
            "{} uses direct `<list>` logic and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator only executes decision-table backed decisions in this slice; direct list decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, item ordering, and nested list expressions while deciding whether this logic can be expressed as one bounded `<decisionTable>`.".to_string(),
            "Do not flatten list items into guessed decision-table rules unless the element-to-clause mapping and element-to-rule mapping are explicit and lossless.".to_string(),
            "If no safe table conversion exists yet, keep the source as a non-executable DMN placeholder and report unsupported `list` execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not rewrite its direct `<list>` into guessed decision-table rules. Only replace the list with one equivalent bounded `<decisionTable>` when the item ordering, nested expressions, and rule mapping are explicit and lossless; otherwise keep the decision non-executable and report unsupported list execution."
        ),
        augment_evidence(json!({
            "decision_id": decision_id,
        }), snapshot, Some(decision_id)),
    )
}
