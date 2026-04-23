use super::decision::{
    decision_required_authority_count, decision_required_decision_count,
    decision_required_input_count, decision_required_knowledge_count,
};
use super::evidence::{augment_evidence, decision_display, root_context};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn unsupported_information_requirement_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let (title, why_it_failed, how_to_fix, repair_guidance, llm_fix_prompt) =
        if decision_required_input_count(decision_id, snapshot) > 0
            && decision_required_decision_count(decision_id, snapshot) == 0
        {
            (
                "DMN decision depends on required input data but has no local decision table",
                format!(
                    "{} uses `<informationRequirement>` with `<requiredInput>` but does not contain a local `<decisionTable>`.",
                    decision_display(decision_id, snapshot)
                ),
                format!(
                    "The bounded evaluator executes one local decision table per decision in this slice; required-input references identify upstream data dependencies, not local executable rules.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-input references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not fabricate decision-table rules or inline upstream input binding semantics only from `<requiredInput>` references unless the missing local logic is explicit and lossless.".to_string(),
                    "If this decision is intentionally only a data-dependency node in a broader DRD, keep it non-executable and report unsupported required-input-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules just from its `<requiredInput>` dependency. Only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported required-input-only execution."
                ),
            )
        } else if decision_required_decision_count(decision_id, snapshot) > 0
            && decision_required_input_count(decision_id, snapshot) == 0
        {
            (
                "DMN decision depends on another decision but has no local decision table",
                format!(
                    "{} uses `<informationRequirement>` with `<requiredDecision>` but does not contain a local `<decisionTable>`.",
                    decision_display(decision_id, snapshot)
                ),
                format!(
                    "The bounded evaluator executes one local decision table per decision in this slice; upstream decision references do not materialize local executable rules automatically.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-decision references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not inline or approximate upstream decision logic only from `<requiredDecision>` references unless the missing local rules are explicit and lossless.".to_string(),
                    "If this decision is intentionally only a decision-dependency node in a broader DRD, keep it non-executable and report unsupported required-decision-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules just from its `<requiredDecision>` dependency. Only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported required-decision-only execution."
                ),
            )
        } else {
            (
                "DMN decision exposes information dependencies but no local decision table",
                format!(
                    "{} uses `<informationRequirement>` edges but does not contain a local `<decisionTable>`.",
                    decision_display(decision_id, snapshot)
                ),
                format!(
                    "The bounded evaluator executes one local decision table per decision in this slice; dependency edges alone do not provide executable decision logic.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and dependency references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not fabricate decision-table rules only from `requiredInput` or `requiredDecision` references unless the missing local logic is explicit and lossless.".to_string(),
                    "If this decision is intentionally only a dependency node in a broader DRD, keep it non-executable and report unsupported information-requirement-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules just from its `<informationRequirement>` edges. Only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported information-requirement-only execution."
                ),
            )
        };

    LintIssue::new(
        "dmn.unsupported_information_requirement_decision",
        title,
        why_it_failed,
        how_to_fix,
        repair_guidance,
        llm_fix_prompt,
        augment_evidence(
            json!({
                "decision_id": decision_id,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

pub(super) fn unsupported_knowledge_requirement_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let (title, why_it_failed, how_to_fix, repair_guidance, llm_fix_prompt) =
        if decision_required_knowledge_count(decision_id, snapshot) > 0 {
            (
                "DMN decision depends on required knowledge but has no local decision table",
                format!(
                    "{} uses `<knowledgeRequirement>` with `<requiredKnowledge>` but does not contain a local `<decisionTable>`.",
                    decision_display(decision_id, snapshot)
                ),
                format!(
                    "The bounded evaluator executes one local decision table per decision in this slice; required-knowledge and business-knowledge-model references do not become local executable logic automatically.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-knowledge references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not inline or approximate business-knowledge-model semantics only from `<requiredKnowledge>` references unless the missing logic is explicit and lossless.".to_string(),
                    "If this decision intentionally depends on broader DMN knowledge models, keep it non-executable and report unsupported required-knowledge-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules from its `<requiredKnowledge>` dependency. Only add one bounded local decision table when the referenced knowledge logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported required-knowledge-only execution."
                ),
            )
        } else {
            (
                "DMN decision depends on knowledge requirements but has no local decision table",
                format!(
                    "{} uses `<knowledgeRequirement>` edges but does not contain a local `<decisionTable>`.",
                    decision_display(decision_id, snapshot)
                ),
                format!(
                    "The bounded evaluator executes one local decision table per decision in this slice; references to external or separate knowledge models do not become local executable logic automatically.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-knowledge references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not inline or approximate business-knowledge-model semantics unless the imported or referenced logic is explicit and lossless.".to_string(),
                    "If this decision intentionally depends on broader DMN knowledge models, keep it non-executable and report unsupported knowledge-requirement-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules from its `<knowledgeRequirement>` edges. Only add one bounded local decision table when the referenced knowledge logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported knowledge-requirement-only execution."
                ),
            )
        };

    LintIssue::new(
        "dmn.unsupported_knowledge_requirement_decision",
        title,
        why_it_failed,
        how_to_fix,
        repair_guidance,
        llm_fix_prompt,
        augment_evidence(
            json!({
                "decision_id": decision_id,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

pub(super) fn unsupported_authority_requirement_decision_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let (title, why_it_failed, how_to_fix, repair_guidance, llm_fix_prompt) =
        if decision_required_authority_count(decision_id, snapshot) > 0 {
            (
                "DMN decision depends on required authority but has no local decision table",
                format!(
                    "{} uses `<authorityRequirement>` with `<requiredAuthority>` but does not contain a local `<decisionTable>`.",
                    decision_display(decision_id, snapshot)
                ),
                format!(
                    "The bounded evaluator executes one local decision table per decision in this slice; required-authority and knowledge-source references do not provide local executable rules by themselves.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-authority references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not fabricate decision-table rules only from `<requiredAuthority>` or `knowledgeSource` metadata unless the missing local logic is explicit and lossless.".to_string(),
                    "If this decision intentionally points at a broader governance or knowledge-source surface, keep it non-executable and report unsupported required-authority-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules from its `<requiredAuthority>` dependency. Only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported required-authority-only execution."
                ),
            )
        } else {
            (
                "DMN decision references authority requirements but has no local decision table",
                format!(
                    "{} uses `<authorityRequirement>` edges but does not contain a local `<decisionTable>`.",
                    decision_display(decision_id, snapshot)
                ),
                format!(
                    "The bounded evaluator executes one local decision table per decision in this slice; authority and knowledge-source references do not provide local executable rules by themselves.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-authority references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not fabricate decision-table rules only from authority or knowledge-source metadata unless the missing local logic is explicit and lossless.".to_string(),
                    "If this decision intentionally points at a broader governance or knowledge-source surface, keep it non-executable and report unsupported authority-requirement-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules from its `<authorityRequirement>` edges. Only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported authority-requirement-only execution."
                ),
            )
        };

    LintIssue::new(
        "dmn.unsupported_authority_requirement_decision",
        title,
        why_it_failed,
        how_to_fix,
        repair_guidance,
        llm_fix_prompt,
        augment_evidence(
            json!({
                "decision_id": decision_id,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}
