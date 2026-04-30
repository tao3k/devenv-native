use super::*;

#[test]
fn bpmn_linter_reports_diagram_interchange_metadata_surface_with_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-bpmn-diagram.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.metadata_di_surface");
    assert!(issue.why_it_failed.contains("round-trip compatibility"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("diagram-interchange metadata")
    );
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(issue.evidence["snapshot"]["diagram_count"], 1);
    assert_eq!(
        issue.evidence["snapshot"]["diagrams"][0]["diagram_id"],
        "Diagram_Main"
    );
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["shape_count"], 2);
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["edge_count"], 1);
}
