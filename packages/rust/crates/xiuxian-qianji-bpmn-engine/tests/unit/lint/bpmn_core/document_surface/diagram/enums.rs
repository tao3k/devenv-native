use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_invalid_di_enum_values_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-enum-values.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_enum");
    assert_eq!(issue.evidence["invalid_enum_count"], 2);
    assert_eq!(
        issue.evidence["invalid_enums"][0]["diagram_id"],
        "Diagram_InvalidDiEnumValues"
    );
    assert_eq!(
        issue.evidence["invalid_enums"][0]["plane_id"],
        "Plane_InvalidDiEnumValues"
    );
    assert_eq!(issue.evidence["invalid_enums"][0]["element"], "BPMNShape");
    assert_eq!(
        issue.evidence["invalid_enums"][0]["element_id"],
        "Shape_Start"
    );
    assert_eq!(
        issue.evidence["invalid_enums"][0]["attribute"],
        "participantBandKind"
    );
    assert_eq!(issue.evidence["invalid_enums"][0]["value"], "top_primary");
    assert_eq!(
        issue.evidence["invalid_enums"][0]["allowed_values"][0],
        "top_initiating"
    );

    assert_eq!(issue.evidence["invalid_enums"][1]["element"], "BPMNEdge");
    assert_eq!(
        issue.evidence["invalid_enums"][1]["element_id"],
        "Edge_StartEnd"
    );
    assert_eq!(
        issue.evidence["invalid_enums"][1]["attribute"],
        "messageVisibleKind"
    );
    assert_eq!(issue.evidence["invalid_enums"][1]["value"], "both");
    assert_eq!(
        issue.evidence["invalid_enums"][1]["allowed_values"][1],
        "non_initiating"
    );
}
