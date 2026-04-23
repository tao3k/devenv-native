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
                    "The bounded evaluator executes one local decision table per decision in this slice; direct same-source `<requiredInput>` edges can only bind an already-supplied local input-data alias when explicit `inputData` and nested `variable` metadata exist, so a decision without its own `<decisionTable>` still has no local executable rules.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-input references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not fabricate decision-table rules or broader upstream input remapping only from `<requiredInput>` references unless the missing local logic is explicit and lossless; the current runtime only supports one bounded same-source input-data alias bind when the source metadata is explicit.".to_string(),
                    "If this decision is intentionally only a data-dependency node in a broader DRD, keep it non-executable and report unsupported required-input-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules or broader input remapping just from its `<requiredInput>` dependency. Preserve reported `decision_snapshot.requirement_references` hrefs, then only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported required-input-only execution."
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
                    "The bounded evaluator executes one local decision table per decision in this slice; it can recurse through direct same-source `<requiredDecision>` edges only after the current decision already contributes local executable rules, so upstream decision references still do not materialize a missing local `<decisionTable>` automatically.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-decision references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not inline or approximate upstream decision logic only from `<requiredDecision>` references unless the missing local rules are explicit and lossless.".to_string(),
                    "If this decision is intentionally only a decision-dependency node in a broader DRD, keep it non-executable and report unsupported required-decision-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules just from its `<requiredDecision>` dependency. Preserve reported `decision_snapshot.requirement_references` hrefs, then only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported required-decision-only execution."
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
                    "The bounded evaluator executes one local decision table per decision in this slice; bounded same-source required-input alias binding and required-decision recursion only extend a decision that already has local executable rules, so dependency edges alone do not provide executable decision logic.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and dependency references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not fabricate decision-table rules or broader DRD dependency semantics only from `requiredInput` or `requiredDecision` references unless the missing local logic is explicit and lossless.".to_string(),
                    "If this decision is intentionally only a dependency node in a broader DRD, keep it non-executable and report unsupported information-requirement-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules just from its `<informationRequirement>` edges. Preserve reported `decision_snapshot.requirement_references` hrefs, then only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported information-requirement-only execution."
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
                    "The bounded evaluator executes one local decision table per decision in this slice; direct same-source `<requiredKnowledge>` edges point at `businessKnowledgeModel` invocables, and the bounded parser now preserves one invocable `variable` / `encapsulatedLogic` placeholder contract, but runtime still does not execute that callable knowledge surface automatically.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-knowledge references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not inline or approximate business-knowledge-model semantics only from `<requiredKnowledge>` references unless the missing invocable contract, callable parameters, and local decision logic are explicit and lossless; the current runtime still does not execute preserved BKM invocable metadata.".to_string(),
                    "If this decision intentionally depends on broader DMN knowledge models, keep it non-executable and report unsupported required-knowledge-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules from its `<requiredKnowledge>` dependency. Preserve reported `decision_snapshot.requirement_references` hrefs, then only add one bounded local decision table when the referenced knowledge logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported required-knowledge-only execution."
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
                    "The bounded evaluator executes one local decision table per decision in this slice; references to external or separate knowledge models still do not become local executable logic automatically, and runtime still does not execute preserved invocable `variable` / `encapsulatedLogic` metadata even for local BKM targets.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and required-knowledge references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not inline or approximate business-knowledge-model semantics unless the referenced invocable contract and local decision logic are explicit and lossless.".to_string(),
                    "If this decision intentionally depends on broader DMN knowledge models, keep it non-executable and report unsupported knowledge-requirement-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules from its `<knowledgeRequirement>` edges. Preserve reported `decision_snapshot.requirement_references` hrefs, then only add one bounded local decision table when the referenced knowledge logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported knowledge-requirement-only execution."
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
                    "The bounded evaluator executes one local decision table per decision in this slice; required-authority, knowledge-source, and any authority-linked decision or input references still do not provide local executable rules by themselves.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and authority-requirement references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not fabricate decision-table rules only from `<requiredAuthority>`, `<requiredDecision>`, `<requiredInput>`, or `knowledgeSource` metadata unless the missing local logic is explicit and lossless.".to_string(),
                    "If this decision intentionally points at a broader governance or knowledge-source surface, keep it non-executable and report unsupported required-authority-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules from its `<authorityRequirement>` edges. Preserve reported `decision_snapshot.requirement_references` hrefs, then only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported required-authority-only execution."
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
                    "The bounded evaluator executes one local decision table per decision in this slice; authority, knowledge-source, and any authority-linked decision or input references still do not provide local executable rules by themselves.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Preserve the existing decision id, name, and authority-requirement references while deciding whether one explicit local `<decisionTable>` exists for this decision.".to_string(),
                    "Do not fabricate decision-table rules only from `<requiredAuthority>`, `<requiredDecision>`, `<requiredInput>`, or `knowledgeSource` metadata unless the missing local logic is explicit and lossless.".to_string(),
                    "If this decision intentionally points at a broader governance or knowledge-source surface, keep it non-executable and report unsupported authority-requirement-only execution.".to_string(),
                ],
                format!(
                    "Inspect decision '{decision_id}' and do not invent local `<decisionTable>` rules from its `<authorityRequirement>` edges. Preserve reported `decision_snapshot.requirement_references` hrefs, then only add one bounded local decision table when the missing logic is explicit and lossless; otherwise keep the decision non-executable and report unsupported authority-requirement-only execution."
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
