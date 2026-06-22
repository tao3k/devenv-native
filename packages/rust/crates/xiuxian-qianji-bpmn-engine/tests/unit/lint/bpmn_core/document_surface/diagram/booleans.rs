use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_invalid_di_boolean_values_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-boolean-values.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_boolean");
    assert_eq!(issue.evidence["invalid_boolean_count"], 3);
    assert_eq!(
        issue.evidence["invalid_booleans"][0]["element"],
        "BPMNShape"
    );
    assert_eq!(
        issue.evidence["invalid_booleans"][0]["element_id"],
        "Shape_Start"
    );
    assert_eq!(
        issue.evidence["invalid_booleans"][0]["attribute"],
        "isHorizontal"
    );
    assert_eq!(issue.evidence["invalid_booleans"][0]["value"], "yes");
    assert_eq!(
        issue.evidence["invalid_booleans"][0]["allowed_values"][0],
        "true"
    );

    assert_eq!(
        issue.evidence["invalid_booleans"][1]["attribute"],
        "isExpanded"
    );
    assert_eq!(issue.evidence["invalid_booleans"][1]["value"], "2");

    assert_eq!(issue.evidence["invalid_booleans"][2]["element"], "Font");
    assert_eq!(
        issue.evidence["invalid_booleans"][2]["path"],
        "definitions/BPMNDiagram/BPMNLabelStyle/Font"
    );
    assert_eq!(issue.evidence["invalid_booleans"][2]["attribute"], "isBold");
    assert_eq!(issue.evidence["invalid_booleans"][2]["value"], "sometimes");
}
