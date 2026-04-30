use super::*;

#[test]
fn bpmn_linter_reports_missing_di_required_attributes_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-di-missing-required-attributes.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.missing_di_required_attribute");
    assert_eq!(issue.evidence["missing_required_attribute_count"], 3);
    assert_eq!(
        issue.evidence["missing_required_attributes"][0]["element"],
        "Bounds"
    );
    assert_eq!(
        issue.evidence["missing_required_attributes"][0]["missing_attribute"],
        "y"
    );
    assert_eq!(
        issue.evidence["missing_required_attributes"][0]["required_attributes"][0],
        "x"
    );
    assert_eq!(
        issue.evidence["missing_required_attributes"][1]["missing_attribute"],
        "height"
    );

    assert_eq!(
        issue.evidence["missing_required_attributes"][2]["element"],
        "waypoint"
    );
    assert_eq!(
        issue.evidence["missing_required_attributes"][2]["path"],
        "definitions/BPMNDiagram/BPMNPlane/BPMNEdge/waypoint"
    );
    assert_eq!(
        issue.evidence["missing_required_attributes"][2]["missing_attribute"],
        "y"
    );
}
