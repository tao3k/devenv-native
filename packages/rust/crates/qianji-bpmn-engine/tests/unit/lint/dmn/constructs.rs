use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
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
fn dmn_linter_reports_literal_expression_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-literal-expression-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_literal_expression_decision");
    assert!(issue.title.contains("literal expression"));
    assert!(
        issue
            .why_it_failed
            .contains("only executes decision-table backed decisions")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not silently approximate"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not rewrite its `<literalExpression>`")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_literal_expression")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_table_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["literal_expression_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["context_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["invocation_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["relation_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definition_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}

#[test]
fn dmn_linter_reports_context_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-context-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_context_decision");
    assert!(issue.title.contains("context logic"));
    assert!(
        issue
            .why_it_failed
            .contains("direct context decisions remain placeholder-only")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not flatten context entries"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not rewrite its direct `<context>`")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_context")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["literal_expression_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["context_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["invocation_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["relation_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definition_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}

#[test]
fn dmn_linter_reports_invocation_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-invocation-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_invocation_decision");
    assert!(issue.title.contains("invocation logic"));
    assert!(
        issue
            .why_it_failed
            .contains("direct invocation decisions remain placeholder-only")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not inline or fabricate invoked logic"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not rewrite its direct `<invocation>`")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_invocation")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["literal_expression_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["context_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["invocation_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["relation_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definition_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}

#[test]
fn dmn_linter_reports_relation_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-relation-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_relation_decision");
    assert!(issue.title.contains("relation logic"));
    assert!(
        issue
            .why_it_failed
            .contains("direct relation decisions remain placeholder-only")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not flatten relation rows"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not rewrite its direct `<relation>`")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_relation")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["literal_expression_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["context_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["invocation_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["relation_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definition_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}
