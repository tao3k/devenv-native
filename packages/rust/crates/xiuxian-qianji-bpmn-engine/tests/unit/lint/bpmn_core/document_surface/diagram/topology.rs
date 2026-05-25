use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_diagram_missing_direct_plane_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-missing-plane.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_plane_topology");
    assert!(
        issue
            .why_it_failed
            .contains("exactly one direct BPMN plane")
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("exactly one direct `BPMNPlane`")
    );
    assert_eq!(issue.evidence["invalid_topology_count"], 1);
    assert_eq!(
        issue.evidence["invalid_topology"][0]["diagram_id"],
        "Diagram_InvalidDiMissingPlane"
    );
    assert_eq!(
        issue.evidence["invalid_topology"][0]["reason"],
        "missing_direct_plane"
    );
    assert_eq!(issue.evidence["invalid_topology"][0]["observed_count"], 0);
}

#[test]
fn bpmn_linter_reports_diagram_multiple_direct_planes_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-multiple-planes.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_plane_topology");
    assert_eq!(issue.evidence["invalid_topology_count"], 1);
    assert_eq!(
        issue.evidence["invalid_topology"][0]["diagram_id"],
        "Diagram_InvalidDiMultiplePlanes"
    );
    assert_eq!(
        issue.evidence["invalid_topology"][0]["reason"],
        "multiple_direct_planes"
    );
    assert_eq!(issue.evidence["invalid_topology"][0]["observed_count"], 2);
    assert_eq!(
        issue.evidence["invalid_topology"][0]["observed_plane_ids"][0],
        "Plane_Main"
    );
    assert_eq!(
        issue.evidence["invalid_topology"][0]["observed_plane_ids"][1],
        "Plane_Secondary"
    );
}

#[test]
fn bpmn_linter_reports_orphan_di_plane_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-orphan-plane.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_plane_topology");
    assert_eq!(issue.evidence["invalid_topology_count"], 1);
    assert_eq!(
        issue.evidence["invalid_topology"][0]["element"],
        "BPMNPlane"
    );
    assert_eq!(
        issue.evidence["invalid_topology"][0]["plane_id"],
        "Plane_Orphan"
    );
    assert_eq!(
        issue.evidence["invalid_topology"][0]["reason"],
        "plane_outside_diagram"
    );
    assert_eq!(
        issue.evidence["invalid_topology"][0]["parent"],
        "definitions"
    );
}
