use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_text_annotation_only_document_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-text-annotation-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_text_annotation_document");
    assert!(issue.title.contains("text annotations"));
    assert!(issue.summary.contains("<textAnnotation>"));
    assert!(
        issue
            .why_it_failed
            .contains("text annotations as descriptive metadata only")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent rules"))
    );
    assert!(issue.llm_fix_prompt.contains(
        "do not fabricate decision-table logic just from top-level `<textAnnotation>` metadata"
    ));
    assert_eq!(issue.evidence["text_annotation_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["text_annotation_count"],
        json!(1)
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}
