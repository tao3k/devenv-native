use super::*;

#[test]
fn bpmn_linter_reports_invalid_bpmndi_namespace_before_di_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-bpmndi-namespace.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_namespace");
    assert!(issue.why_it_failed.contains("Native BPMN"));
    assert_eq!(issue.evidence["invalid_namespace_count"], 5);
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["element"],
        "BPMNDiagram"
    );
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["element_id"],
        "Diagram_InvalidDiBpmndiNamespace"
    );
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["namespace_uri"],
        "https://example.com/not-bpmndi"
    );
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["expected_namespace_uri"],
        "http://www.omg.org/spec/BPMN/20100524/DI"
    );
}

#[test]
fn bpmn_linter_reports_invalid_dc_namespace_before_completeness_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-dc-namespace.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_namespace");
    assert_eq!(issue.evidence["invalid_namespace_count"], 2);
    assert_eq!(issue.evidence["invalid_namespaces"][0]["element"], "Bounds");
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["namespace_uri"],
        "https://example.com/not-dc"
    );
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["expected_namespace_uri"],
        "http://www.omg.org/spec/DD/20100524/DC"
    );
}

#[test]
fn bpmn_linter_reports_invalid_di_waypoint_namespace_before_completeness_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-waypoint-namespace.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_namespace");
    assert_eq!(issue.evidence["invalid_namespace_count"], 2);
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["element"],
        "waypoint"
    );
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["namespace_uri"],
        "https://example.com/not-di"
    );
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["expected_namespace_uri"],
        "http://www.omg.org/spec/DD/20100524/DI"
    );
}
