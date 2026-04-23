use super::super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_decision_service_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-decision-service-is-collapsed-20180521.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_decision_service");
    assert!(issue.title.contains("decision service"));
    assert!(
        issue
            .why_it_failed
            .contains("does not execute `decisionService`")
    );
    assert!(issue.why_it_failed.contains("name 'Decision Service 1'"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent decision-table rules"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not fabricate decision-table logic")
    );
    assert!(issue.why_it_failed.contains(
        "DMNDecisionServiceDividerLine 2 waypoint(s) [di:waypoint x '0', y '210'; di:waypoint x '906', y '210']"
    ));
    assert!(issue.why_it_failed.contains("isCollapsed false"));
    assert_eq!(
        issue.evidence["document_root"]["decision_service_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["document_root"]["decision_services"][0]["decision_service_id"],
        json!("DecisionService_1")
    );
    assert_eq!(
        issue.evidence["document_root"]["decision_services"][0]["name"],
        json!("Decision Service 1")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"]["bounds"]
            ["x"],
        json!("354")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["is_collapsed"],
        json!(false)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["decision_service_divider_line"]
            ["waypoints"][0]["x"],
        json!("0")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["decision_service_divider_line"]
            ["waypoints"][1]["y"],
        json!("210")
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}

#[test]
fn dmn_linter_surfaces_decision_service_reference_placeholders() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-decision-service-references-20180521.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_decision_service");
    assert!(
        issue
            .why_it_failed
            .contains("outputDecision href '#Decision_approval'")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Preserve any reported `outputDecision`"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("Preserve reported `document_root.decision_services` references")
    );
    let decision_service = &issue.evidence["document_root"]["decision_services"][0];
    assert_eq!(
        decision_service["decision_service_id"],
        json!("DecisionService_credit")
    );
    assert_eq!(
        decision_service["output_decisions"][0]["reference_kind"],
        json!("outputDecision")
    );
    assert_eq!(
        decision_service["output_decisions"][0]["href"],
        json!("#Decision_approval")
    );
    assert_eq!(
        decision_service["encapsulated_decisions"][0]["reference_kind"],
        json!("encapsulatedDecision")
    );
    assert_eq!(
        decision_service["encapsulated_decisions"][0]["href"],
        json!("#Decision_risk_score")
    );
    assert_eq!(
        decision_service["input_decisions"][0]["reference_kind"],
        json!("inputDecision")
    );
    assert_eq!(
        decision_service["input_decisions"][0]["href"],
        json!("#Decision_prior_risk")
    );
    assert_eq!(
        decision_service["input_data"][0]["reference_kind"],
        json!("inputData")
    );
    assert_eq!(
        decision_service["input_data"][0]["href"],
        json!("#InputData_application")
    );
}
