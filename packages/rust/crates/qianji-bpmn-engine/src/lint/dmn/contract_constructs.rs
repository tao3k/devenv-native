use super::evidence::{augment_evidence, decision_display, root_context};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn unsupported_context_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::from_parts(
        "dmn.unsupported_context_decision",
        "DMN decision uses context logic outside the executable subset",
        format!(
            "{} uses direct `<context>` logic that was not accepted by the bounded context parser.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator executes direct contexts only when every entry has one supported literal-expression body, optional variable metadata, and any unnamed result entry is final; broader context decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, name, and context-entry ordering.".to_string(),
            "If every entry can be represented as a bounded literal-expression item, keep the direct `<context>` and rewrite only the unsupported entries.".to_string(),
            "Do not flatten broader context logic into guessed decision-table rules unless the entry-to-clause mapping and every entry-to-rule mapping are explicit and lossless.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and rewrite its direct `<context>` only to supported bounded context entries. Do not replace broader context logic with guessed rules; use an equivalent bounded `<decisionTable>` only when the entry ordering and rule mapping are explicit and lossless."
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
    LintIssue::from_parts(
        "dmn.unsupported_invocation_decision",
        "DMN decision uses invocation logic instead of a decision table",
        format!(
            "{} uses direct `<invocation>` logic and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator does not execute direct invocation decisions yet; invocation function-expression and binding metadata are preserved only as non-executable snapshot evidence.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, name, and invocation bindings while deciding whether this logic can be expressed as one bounded `<decisionTable>`.".to_string(),
            "Use the `decision_snapshot.invocations` evidence to preserve the invoked expression and each binding parameter/argument pair.".to_string(),
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
    LintIssue::from_parts(
        "dmn.unsupported_literal_expression_decision",
        "DMN decision uses literal expression logic outside the executable subset",
        format!(
            "{} uses `<literalExpression>` logic that was not accepted by the bounded direct-expression parser.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator executes direct literal expressions only when they fit the supported literal/path/simple numeric-path subset; broader literal-expression decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id and name while deciding whether this logic can be expressed as one bounded `<decisionTable>`.".to_string(),
            "If the expression is a constant string, quote it explicitly; if it reads input data, reduce it to one variable path; if it performs arithmetic, reduce it to one whitespace-delimited `path + number` or `path - number` operation.".to_string(),
            "Do not silently approximate broader FEEL expressions; convert them only if the equivalent decision-table rules are explicit and lossless.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and rewrite its `<literalExpression>` only to a supported bounded literal/path/simple numeric-path form. Do not replace it with guessed rules; use an equivalent bounded `<decisionTable>` only when the rule mapping is explicit and lossless."
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
    LintIssue::from_parts(
        "dmn.unsupported_relation_decision",
        "DMN decision uses relation logic outside the executable subset",
        format!(
            "{} uses direct `<relation>` logic that was not accepted by the bounded relation parser.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator executes direct relations only when every row cell is a supported literal expression and every row matches the relation column count; broader relation decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, relation columns, and row ordering.".to_string(),
            "If every row cell can be represented as a bounded literal expression, keep the direct `<relation>` and rewrite only the unsupported cells.".to_string(),
            "Do not flatten broader relation logic into guessed decision-table rules unless the column-to-clause mapping and every row-to-rule mapping are explicit and lossless.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and rewrite its direct `<relation>` only to supported bounded relation rows and literal-expression cells. Do not replace broader relation logic with guessed rules; use an equivalent bounded `<decisionTable>` only when the columns, rows, and rule mapping are explicit and lossless."
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
    LintIssue::from_parts(
        "dmn.unsupported_function_definition_decision",
        "DMN decision uses function definition logic instead of a decision table",
        format!(
            "{} uses direct `<functionDefinition>` logic and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator does not execute direct functionDefinition decisions yet; function kind, formal-parameter, and body literal-expression metadata are preserved only as non-executable snapshot evidence.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, formal parameters, and function body while deciding whether this logic can be expressed as one bounded `<decisionTable>`.".to_string(),
            "Use the `decision_snapshot.function_definitions` evidence to preserve the function kind, each formal parameter, and the direct body literal-expression.".to_string(),
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
    LintIssue::from_parts(
        "dmn.unsupported_list_decision",
        "DMN decision uses list logic outside the executable subset",
        format!(
            "{} uses direct `<list>` logic that was not accepted by the bounded direct-list parser.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator executes direct lists only when every direct child is a supported literal-expression item; broader list decisions remain placeholder-only.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id and list item ordering.".to_string(),
            "If every item can be represented as a bounded literal-expression item, keep the direct `<list>` and rewrite only the unsupported items.".to_string(),
            "Do not flatten broader list logic into guessed decision-table rules unless the element-to-clause mapping and element-to-rule mapping are explicit and lossless.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and rewrite its direct `<list>` only to supported bounded literal-expression items. Do not replace broader list logic with guessed rules; use an equivalent bounded `<decisionTable>` only when the item ordering and rule mapping are explicit and lossless."
        ),
        augment_evidence(json!({
            "decision_id": decision_id,
        }), snapshot, Some(decision_id)),
    )
}
