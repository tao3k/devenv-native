use crate::lint::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_accepts_valid_multi_decision_source() {
    let report = lint_dmn_source(&dmn_fixture_source("multiple-decisions.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn dmn_linter_reports_unsupported_unary_test_with_llm_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("invalid-unsupported-unary-test.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_unary_test");
    assert!(issue.summary.contains("duration(\"P1.5Y\")"));
    assert!(issue.why_it_failed.contains("duration(\"PT1,5H\")"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("fractional year-month forms"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("duration(\"PT1,5M\") <= ? < duration(\"PT2,75M\")")
    );
}

#[test]
fn dmn_linter_reports_missing_decision_table_with_document_snapshot_context() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "invalid-missing-decision-table-generic.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.missing_decision_table");
    assert!(issue.summary.contains("Generic Missing Table Decision"));
    assert!(issue.why_it_failed.contains("20191111"));
    assert_eq!(
        issue.evidence["document_root"]["model_namespace_uri"],
        json!("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        issue.evidence["document_root"]["model_version_hint"],
        json!("20191111")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_missing_decision_table_generic")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["allowed_answers_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_maker_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_owner_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_table_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["information_requirement_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["knowledge_requirement_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["authority_requirement_count"],
        json!(0)
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
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definition_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}
