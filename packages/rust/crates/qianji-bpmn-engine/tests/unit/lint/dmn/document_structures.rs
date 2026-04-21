use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_association_only_document_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-association-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_association_document");
    assert!(issue.title.contains("associations"));
    assert!(issue.summary.contains("<association>"));
    assert!(
        issue
            .why_it_failed
            .contains("associations as document-structure metadata only")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent dependencies"))
    );
    assert!(issue.llm_fix_prompt.contains(
        "do not fabricate decision-table logic just from top-level `<association>` metadata"
    ));
    assert_eq!(issue.evidence["association_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["association_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["document_root"]["element_collection_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}

#[test]
fn dmn_linter_reports_element_collection_only_document_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-element-collection-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_element_collection_document");
    assert!(issue.title.contains("element collections"));
    assert!(issue.summary.contains("<elementCollection>"));
    assert!(
        issue
            .why_it_failed
            .contains("element collections as structural metadata only")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent grouped members"))
    );
    assert!(issue.llm_fix_prompt.contains(
        "do not fabricate decision-table logic just from top-level `<elementCollection>` metadata"
    ));
    assert_eq!(issue.evidence["element_collection_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["element_collection_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["document_root"]["association_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}
