use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_group_only_document_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("metadata-only-group-20191111.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_group_document");
    assert!(issue.title.contains("group artifacts"));
    assert!(issue.summary.contains("<group>"));
    assert!(
        issue
            .why_it_failed
            .contains("groups as non-executable structural metadata only")
    );
    assert!(issue.why_it_failed.contains("Manual Review Cluster"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent rules"))
    );
    assert!(
        issue.llm_fix_prompt.contains(
            "do not fabricate decision-table logic just from top-level `<group>` metadata"
        )
    );
    assert_eq!(issue.evidence["group_count"], json!(1));
    assert_eq!(issue.evidence["document_root"]["group_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["groups"][0]["group_id"],
        json!("Group_manual_review_cluster")
    );
    assert_eq!(
        issue.evidence["document_root"]["groups"][0]["name"],
        json!("Manual Review Cluster")
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}
