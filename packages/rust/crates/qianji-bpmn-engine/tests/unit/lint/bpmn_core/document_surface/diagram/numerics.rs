use super::*;

#[test]
fn bpmn_linter_reports_invalid_di_numeric_values_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-numeric-values.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_numeric");
    assert_eq!(issue.evidence["invalid_numeric_count"], 5);
    assert_eq!(
        issue.evidence["invalid_numerics"][0]["element"],
        "BPMNDiagram"
    );
    assert_eq!(
        issue.evidence["invalid_numerics"][0]["element_id"],
        "Diagram_InvalidDiNumericValues"
    );
    assert_eq!(
        issue.evidence["invalid_numerics"][0]["attribute"],
        "resolution"
    );
    assert_eq!(issue.evidence["invalid_numerics"][0]["value"], "dense");
    assert_eq!(
        issue.evidence["invalid_numerics"][0]["expected"],
        "finite_xsd_double"
    );

    assert_eq!(issue.evidence["invalid_numerics"][1]["element"], "Bounds");
    assert_eq!(issue.evidence["invalid_numerics"][1]["attribute"], "y");
    assert_eq!(issue.evidence["invalid_numerics"][1]["value"], "top");
    assert_eq!(issue.evidence["invalid_numerics"][2]["attribute"], "width");
    assert_eq!(issue.evidence["invalid_numerics"][2]["value"], "NaN");

    assert_eq!(issue.evidence["invalid_numerics"][3]["element"], "waypoint");
    assert_eq!(issue.evidence["invalid_numerics"][3]["attribute"], "x");
    assert_eq!(issue.evidence["invalid_numerics"][3]["value"], "far");

    assert_eq!(issue.evidence["invalid_numerics"][4]["element"], "Font");
    assert_eq!(
        issue.evidence["invalid_numerics"][4]["path"],
        "definitions/BPMNDiagram/BPMNLabelStyle/Font"
    );
    assert_eq!(issue.evidence["invalid_numerics"][4]["attribute"], "size");
    assert_eq!(issue.evidence["invalid_numerics"][4]["value"], "huge");
}
