use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_artifact_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-artifacts.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_collaboration_surface");
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(issue.evidence["snapshot"]["artifact_association_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["artifact_group_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["text_annotation_count"], 1);
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["associations"][0]["source_ref"],
        "TextAnnotation_Collaboration"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["groups"][0]["category_value_ref"],
        "CategoryValue_ManualReview"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["text_annotations"][0]["text"],
        "Review note from collaboration scope"
    );
}
