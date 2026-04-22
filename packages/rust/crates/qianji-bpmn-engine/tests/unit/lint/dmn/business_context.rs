use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_organization_unit_only_document_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-organization-unit-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_organization_unit_document");
    assert!(issue.title.contains("organization-unit business context"));
    assert!(issue.summary.contains("<organizationUnit>"));
    assert!(
        issue
            .why_it_failed
            .contains("organization-unit declarations as governance metadata only")
    );
    assert!(issue.why_it_failed.contains("Credit Risk Committee"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent approval rules"))
    );
    assert!(issue.llm_fix_prompt.contains(
        "do not fabricate decision-table logic just from top-level `<organizationUnit>` metadata"
    ));
    assert_eq!(issue.evidence["organization_unit_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["organization_unit_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["document_root"]["organization_units"][0]["organization_unit_id"],
        json!("OrganizationUnit_credit_risk")
    );
    assert_eq!(
        issue.evidence["document_root"]["organization_units"][0]["name"],
        json!("Credit Risk Committee")
    );
    assert_eq!(
        issue.evidence["document_root"]["performance_indicator_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}

#[test]
fn dmn_linter_reports_performance_indicator_only_document_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-performance-indicator-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_performance_indicator_document");
    assert!(
        issue
            .title
            .contains("performance-indicator business context")
    );
    assert!(issue.summary.contains("<performanceIndicator>"));
    assert!(
        issue
            .why_it_failed
            .contains("performance indicators as monitoring metadata only")
    );
    assert!(issue.why_it_failed.contains("Auto Adjudication Rate"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent thresholds"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not fabricate decision-table logic just from top-level `<performanceIndicator>` metadata")
    );
    assert_eq!(issue.evidence["performance_indicator_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["organization_unit_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["document_root"]["performance_indicator_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["document_root"]["performance_indicators"][0]["performance_indicator_id"],
        json!("PerformanceIndicator_auto_adjudication_rate")
    );
    assert_eq!(
        issue.evidence["document_root"]["performance_indicators"][0]["name"],
        json!("Auto Adjudication Rate")
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}
