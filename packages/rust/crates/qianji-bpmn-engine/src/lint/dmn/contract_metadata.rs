use super::decision::{decision_maker_count, decision_owner_count};
use super::evidence::{augment_evidence, decision_display, root_context};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn unsupported_allowed_answers_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::from_parts(
        "dmn.unsupported_allowed_answers_decision",
        "DMN decision contains allowed answers metadata but no decision table",
        format!(
            "{} contains direct `<allowedAnswers>` metadata and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator treats decision-owned `<allowedAnswers>` as non-executable output metadata only; it does not derive executable decision logic without one local `<decisionTable>`.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, name, and allowed-answer text while deciding whether this metadata can be paired with one bounded `<decisionTable>`.".to_string(),
            "Do not invent rules, hit policies, or output coercions just from `<allowedAnswers>` metadata unless the missing decision-table mapping is explicit and lossless.".to_string(),
            "If the decision is intentionally metadata-only, keep it as a non-executable DMN placeholder and report unsupported `allowedAnswers`-only execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not fabricate decision-table rules just from its `<allowedAnswers>` metadata. Only add one bounded `<decisionTable>` when the mapping from allowed answers to explicit rules and outputs is lossless; otherwise keep the decision non-executable and report unsupported allowed-answers-only execution."
        ),
        augment_evidence(
            json!({
                "decision_id": decision_id,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

pub(super) fn unsupported_decision_maker_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let decision_maker_count = decision_maker_count(decision_id, snapshot);
    let noun = if decision_maker_count == 1 {
        "reference"
    } else {
        "references"
    };
    LintIssue::from_parts(
        "dmn.unsupported_decision_maker_decision",
        "DMN decision contains decision-maker metadata but no decision table",
        format!(
            "{} contains {decision_maker_count} direct `<decisionMaker>` {noun} and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator treats direct `<decisionMaker>` references as non-executable governance metadata only; they do not become executable decision logic without one local `<decisionTable>`.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, name, and decision-maker references while deciding whether this governance metadata can be paired with one bounded `<decisionTable>`.".to_string(),
            "Do not invent rules, ownership semantics, or output behavior just from `<decisionMaker>` metadata unless the missing decision-table mapping is explicit and lossless.".to_string(),
            "If the decision is intentionally governance-only, keep it as a non-executable DMN placeholder and report unsupported `decisionMaker`-only execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not fabricate decision-table rules just from its `<decisionMaker>` metadata. Only add one bounded `<decisionTable>` when the mapping from those decision-maker references to explicit local rules is lossless; otherwise keep the decision non-executable and report unsupported decision-maker-only execution."
        ),
        augment_evidence(
            json!({
                "decision_id": decision_id,
                "decision_maker_count": decision_maker_count,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

pub(super) fn unsupported_mixed_decision_governance_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let decision_maker_count = decision_maker_count(decision_id, snapshot);
    let decision_owner_count = decision_owner_count(decision_id, snapshot);
    let maker_noun = if decision_maker_count == 1 {
        "reference"
    } else {
        "references"
    };
    let owner_noun = if decision_owner_count == 1 {
        "reference"
    } else {
        "references"
    };
    LintIssue::from_parts(
        "dmn.unsupported_mixed_decision_governance_decision",
        "DMN decision contains maker and owner metadata but no decision table",
        format!(
            "{} contains {decision_maker_count} direct `<decisionMaker>` {maker_noun}, {decision_owner_count} direct `<decisionOwner>` {owner_noun}, and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator treats combined `<decisionMaker>` and `<decisionOwner>` references as governance metadata only; they do not become executable decision logic without one local `<decisionTable>`.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, name, decision-maker references, and decision-owner references while deciding whether this governance metadata can be paired with one bounded `<decisionTable>`.".to_string(),
            "Do not invent rules, approval routing, ownership semantics, or output behavior just from combined `<decisionMaker>` and `<decisionOwner>` metadata unless the missing decision-table mapping is explicit and lossless.".to_string(),
            "If the decision is intentionally governance-only, keep it as a non-executable DMN placeholder and report unsupported mixed decision governance execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not fabricate decision-table rules just from its combined `<decisionMaker>` and `<decisionOwner>` metadata. Only add one bounded `<decisionTable>` when the mapping from those governance references to explicit local rules is lossless; otherwise keep the decision non-executable and report unsupported mixed decision governance execution."
        ),
        augment_evidence(
            json!({
                "decision_id": decision_id,
                "decision_maker_count": decision_maker_count,
                "decision_owner_count": decision_owner_count,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

pub(super) fn unsupported_decision_owner_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let decision_owner_count = decision_owner_count(decision_id, snapshot);
    let noun = if decision_owner_count == 1 {
        "reference"
    } else {
        "references"
    };
    LintIssue::from_parts(
        "dmn.unsupported_decision_owner_decision",
        "DMN decision contains decision-owner metadata but no decision table",
        format!(
            "{} contains {decision_owner_count} direct `<decisionOwner>` {noun} and does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator treats direct `<decisionOwner>` references as non-executable governance metadata only; they do not become executable decision logic without one local `<decisionTable>`.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, name, and decision-owner references while deciding whether this governance metadata can be paired with one bounded `<decisionTable>`.".to_string(),
            "Do not invent rules, ownership semantics, or output behavior just from `<decisionOwner>` metadata unless the missing decision-table mapping is explicit and lossless.".to_string(),
            "If the decision is intentionally governance-only, keep it as a non-executable DMN placeholder and report unsupported `decisionOwner`-only execution.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and do not fabricate decision-table rules just from its `<decisionOwner>` metadata. Only add one bounded `<decisionTable>` when the mapping from those decision-owner references to explicit local rules is lossless; otherwise keep the decision non-executable and report unsupported decision-owner-only execution."
        ),
        augment_evidence(
            json!({
                "decision_id": decision_id,
                "decision_owner_count": decision_owner_count,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

pub(super) fn generic_missing_decision_table_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::from_parts(
        "dmn.missing_decision_table",
        "DMN decision has no decision table",
        format!(
            "{} does not contain a `<decisionTable>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator only understands decision-table backed decisions in this slice.{}",
            root_context(snapshot)
        ),
        vec![
            "Add exactly one `<decisionTable>` under the decision.".to_string(),
            "Move inputs, outputs, and rules inside that table rather than leaving them at the decision level.".to_string(),
        ],
        format!(
            "Repair decision '{decision_id}' by adding exactly one `<decisionTable>` and placing all input, output, and rule clauses inside it."
        ),
        augment_evidence(json!({
            "decision_id": decision_id,
        }), snapshot, Some(decision_id)),
    )
}
