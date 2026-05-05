use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_incomplete_di_shape_bounds_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-shape-missing-bounds.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.incomplete_di_surface");
    assert!(issue.why_it_failed.contains("round-trip"));
    assert_eq!(issue.evidence["incomplete_surface_count"], 1);
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["diagram_id"],
        "Diagram_InvalidDiShapeMissingBounds"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["plane_id"],
        "Plane_InvalidDiShapeMissingBounds"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["element"],
        "BPMNShape"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["element_id"],
        "Shape_Start"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["missing"],
        "dc:Bounds"
    );
}

#[test]
fn bpmn_linter_reports_incomplete_di_edge_waypoints_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-di-edge-missing-waypoints.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.incomplete_di_surface");
    assert_eq!(issue.evidence["incomplete_surface_count"], 1);
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["diagram_id"],
        "Diagram_InvalidDiEdgeMissingWaypoints"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["element"],
        "BPMNEdge"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["element_id"],
        "Edge_StartEnd"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["missing"],
        "di:waypoint[2]"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["observed_count"],
        1
    );
}

#[test]
fn bpmn_linter_reports_incomplete_di_label_style_font_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-di-label-style-missing-font.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.incomplete_di_surface");
    assert!(issue.why_it_failed.contains("round-trip"));
    assert_eq!(issue.evidence["incomplete_surface_count"], 1);
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["diagram_id"],
        "Diagram_InvalidDiLabelStyleMissingFont"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["element"],
        "BPMNLabelStyle"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["element_id"],
        "Style_MissingFont"
    );
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["missing"],
        "dc:Font"
    );
}
