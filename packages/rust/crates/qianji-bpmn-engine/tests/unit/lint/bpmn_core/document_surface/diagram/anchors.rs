use super::*;

#[test]
fn bpmn_linter_reports_di_plane_missing_semantic_anchor_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-plane-missing-anchor.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.missing_di_semantic_anchor");
    assert!(
        issue
            .why_it_failed
            .contains("plane, shape, and edge should declare")
    );
    assert!(issue.llm_fix_prompt.contains("every BPMN DI plane"));
    assert_eq!(issue.evidence["missing_anchor_count"], 1);
    assert_eq!(
        issue.evidence["missing_anchors"][0]["diagram_id"],
        "Diagram_InvalidDiPlaneMissingAnchor"
    );
    assert_eq!(
        issue.evidence["missing_anchors"][0]["plane_id"],
        "Plane_InvalidDiPlaneMissingAnchor"
    );
    assert_eq!(issue.evidence["missing_anchors"][0]["element"], "BPMNPlane");
    assert_eq!(
        issue.evidence["missing_anchors"][0]["missing"],
        "bpmnElement"
    );
}

#[test]
fn bpmn_linter_reports_di_shape_missing_semantic_anchor_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-shape-missing-anchor.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.missing_di_semantic_anchor");
    assert_eq!(issue.evidence["missing_anchor_count"], 1);
    assert_eq!(issue.evidence["missing_anchors"][0]["element"], "BPMNShape");
    assert_eq!(
        issue.evidence["missing_anchors"][0]["element_id"],
        "Shape_Start"
    );
    assert_eq!(
        issue.evidence["missing_anchors"][0]["expected_scope"],
        "semantic_bpmn_id"
    );
}

#[test]
fn bpmn_linter_reports_di_edge_missing_semantic_anchor_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-edge-missing-anchor.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.missing_di_semantic_anchor");
    assert_eq!(issue.evidence["missing_anchor_count"], 1);
    assert_eq!(issue.evidence["missing_anchors"][0]["element"], "BPMNEdge");
    assert_eq!(
        issue.evidence["missing_anchors"][0]["element_id"],
        "Edge_StartEnd"
    );
    assert_eq!(
        issue.evidence["missing_anchors"][0]["missing"],
        "bpmnElement"
    );
}
