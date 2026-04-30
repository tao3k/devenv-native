use super::*;

#[test]
fn bpmn_linter_reports_di_shape_anchor_kind_mismatch_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-shape-anchor-kind.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_anchor_kind");
    assert!(issue.why_it_failed.contains("metadata-only"));
    assert_eq!(issue.evidence["invalid_anchor_kind_count"], 1);
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["diagram_id"],
        "Diagram_InvalidDiShapeAnchorKind"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["element"],
        "BPMNShape"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["element_id"],
        "Shape_Start"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["reference"],
        "flow_start_end"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["actual_semantic_tag"],
        "sequenceFlow"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["expected_anchor_kind"],
        "node_or_artifact"
    );
}

#[test]
fn bpmn_linter_reports_di_edge_anchor_kind_mismatch_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-edge-anchor-kind.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_anchor_kind");
    assert_eq!(issue.evidence["invalid_anchor_kind_count"], 1);
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["diagram_id"],
        "Diagram_InvalidDiEdgeAnchorKind"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["element"],
        "BPMNEdge"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["element_id"],
        "Edge_StartReview"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["reference"],
        "review"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["actual_semantic_tag"],
        "serviceTask"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["expected_anchor_kind"],
        "flow_or_association"
    );
}

#[test]
fn bpmn_linter_reports_di_plane_anchor_kind_mismatch_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-plane-anchor-kind.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_anchor_kind");
    assert_eq!(issue.evidence["invalid_anchor_kind_count"], 1);
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["diagram_id"],
        "Diagram_InvalidDiPlaneAnchorKind"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["element"],
        "BPMNPlane"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["element_id"],
        "Plane_InvalidDiPlaneAnchorKind"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["reference"],
        "start"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["actual_semantic_tag"],
        "startEvent"
    );
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["expected_anchor_kind"],
        "diagram_root"
    );
}
