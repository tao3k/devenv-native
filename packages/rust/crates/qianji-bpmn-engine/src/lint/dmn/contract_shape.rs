use super::contract_constructs::{
    unsupported_context_decision_issue, unsupported_function_definition_decision_issue,
    unsupported_invocation_decision_issue, unsupported_list_decision_issue,
    unsupported_literal_expression_decision_issue, unsupported_relation_decision_issue,
};
use super::contract_metadata::{
    generic_missing_decision_table_issue, unsupported_allowed_answers_decision_issue,
    unsupported_decision_maker_decision_issue, unsupported_decision_owner_decision_issue,
    unsupported_mixed_decision_governance_decision_issue,
};
use super::contract_requirements::{
    unsupported_authority_requirement_decision_issue,
    unsupported_information_requirement_decision_issue,
    unsupported_knowledge_requirement_decision_issue,
};
use super::contract_subset::{
    issue_from_dmn_expression_subset_error, issue_from_dmn_hit_policy_error,
};
use super::decision::{
    decision_has_allowed_answers, decision_has_authority_requirement, decision_has_context,
    decision_has_function_definition, decision_has_information_requirement,
    decision_has_invocation, decision_has_knowledge_requirement, decision_has_list,
    decision_has_literal_expression, decision_has_mixed_decision_governance,
    decision_has_only_decision_maker, decision_has_only_decision_owner, decision_has_relation,
};
use super::evidence::{augment_evidence, decision_display, root_context};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn issue_from_dmn_contract_error(
    error: &BpmnEngineError,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    issue_from_dmn_table_shape_error(error, snapshot)
        .or_else(|| issue_from_dmn_hit_policy_error(error, snapshot))
        .or_else(|| issue_from_dmn_expression_subset_error(error, snapshot))
}

pub(super) fn issue_from_dmn_table_shape_error(
    error: &BpmnEngineError,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnsupportedDmnDecisionCount { source_id, count } => LintIssue::new(
            "dmn.unsupported_decision_count",
            "Exact-one DMN compatibility parser encountered multiple decisions",
            format!(
                "Source '{source_id}' contains {count} decisions, which is valid for the plural DMN parser but not for the exact-one compatibility wrapper."
            ),
            format!(
                "The active lint path accepts multi-decision DMN sources, but the exact-one compatibility parser still rejects them to protect single-decision callers.{}",
                root_context(snapshot)
            ),
            vec![
                "Call the plural DMN parser when the source intentionally contains multiple decisions.".to_string(),
                "Keep the exact-one wrapper only for callers that truly require one decision artifact.".to_string(),
            ],
            format!(
                "If source '{source_id}' intentionally contains multiple `<decision>` elements, route it through the plural DMN parser instead of the exact-one compatibility wrapper. Only split the file if a downstream caller still requires exactly one decision artifact."
            ),
            augment_evidence(json!({
                "source_id": source_id,
                "decision_count": count,
            }), snapshot, None),
        ),
        BpmnEngineError::MissingDmnDecisionTable { decision_id } => {
            missing_dmn_decision_table_issue(decision_id, snapshot)
        }
        BpmnEngineError::UnsupportedDmnDecisionTableCount { decision_id, count } => {
            LintIssue::new(
                "dmn.unsupported_decision_table_count",
                "DMN decision has too many decision tables",
                format!(
                    "{} contains {count} decision tables, but the bounded contract expects exactly 1.",
                    decision_display(decision_id, snapshot)
                ),
                format!(
                    "The current evaluator resolves one decision to one table so adapter wiring stays deterministic.{}",
                    root_context(snapshot)
                ),
                vec![
                    "Keep one decision table in the file and move extra tables into separate decisions or files.".to_string(),
                    "Do not merge unrelated tables if that would change rule meaning.".to_string(),
                ],
                format!(
                    "Edit decision '{decision_id}' so it contains exactly one `<decisionTable>`. Split extra tables into separate decisions or files instead of forcing a lossy merge."
                ),
                augment_evidence(json!({
                    "decision_id": decision_id,
                    "table_count": count,
                }), snapshot, Some(decision_id)),
            )
        }
        _ => return None,
    })
}

pub(super) fn missing_dmn_decision_table_issue(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    if decision_has_context(decision_id, snapshot) {
        unsupported_context_decision_issue(decision_id, snapshot)
    } else if decision_has_invocation(decision_id, snapshot) {
        unsupported_invocation_decision_issue(decision_id, snapshot)
    } else if decision_has_relation(decision_id, snapshot) {
        unsupported_relation_decision_issue(decision_id, snapshot)
    } else if decision_has_function_definition(decision_id, snapshot) {
        unsupported_function_definition_decision_issue(decision_id, snapshot)
    } else if decision_has_list(decision_id, snapshot) {
        unsupported_list_decision_issue(decision_id, snapshot)
    } else if decision_has_literal_expression(decision_id, snapshot) {
        unsupported_literal_expression_decision_issue(decision_id, snapshot)
    } else if decision_has_information_requirement(decision_id, snapshot) {
        unsupported_information_requirement_decision_issue(decision_id, snapshot)
    } else if decision_has_knowledge_requirement(decision_id, snapshot) {
        unsupported_knowledge_requirement_decision_issue(decision_id, snapshot)
    } else if decision_has_authority_requirement(decision_id, snapshot) {
        unsupported_authority_requirement_decision_issue(decision_id, snapshot)
    } else if decision_has_allowed_answers(decision_id, snapshot) {
        unsupported_allowed_answers_decision_issue(decision_id, snapshot)
    } else if decision_has_mixed_decision_governance(decision_id, snapshot) {
        unsupported_mixed_decision_governance_decision_issue(decision_id, snapshot)
    } else if decision_has_only_decision_maker(decision_id, snapshot) {
        unsupported_decision_maker_decision_issue(decision_id, snapshot)
    } else if decision_has_only_decision_owner(decision_id, snapshot) {
        unsupported_decision_owner_decision_issue(decision_id, snapshot)
    } else {
        generic_missing_decision_table_issue(decision_id, snapshot)
    }
}
