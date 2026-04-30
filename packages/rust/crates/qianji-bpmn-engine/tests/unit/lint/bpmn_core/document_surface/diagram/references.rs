use super::*;

#[test]
fn bpmn_linter_reports_invalid_di_semantic_reference_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-reference.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_reference");
    assert!(issue.why_it_failed.contains("round-trip compatibility"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("every BPMN DI `bpmnElement` reference")
    );
    assert_eq!(issue.evidence["invalid_reference_count"], 1);
    assert_eq!(
        issue.evidence["invalid_references"][0]["diagram_id"],
        "Diagram_InvalidDiReference"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["element"],
        "BPMNShape"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["element_id"],
        "Shape_Review"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["reference"],
        "missing_review"
    );
    assert_eq!(issue.evidence["snapshot"]["diagram_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["shape_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["edge_count"], 1);
}

#[test]
fn bpmn_linter_reports_invalid_di_edge_endpoint_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-edge-reference.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_reference");
    assert!(issue.llm_fix_prompt.contains("every DI-local reference"));
    assert_eq!(issue.evidence["invalid_reference_count"], 1);
    assert_eq!(
        issue.evidence["invalid_references"][0]["diagram_id"],
        "Diagram_InvalidDiEdgeReference"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["plane_id"],
        "Plane_InvalidDiEdgeReference"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["element"],
        "BPMNEdge"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["element_id"],
        "Edge_StartEnd"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["attribute"],
        "sourceElement"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["reference"],
        "Missing_StartShape"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["expected_scope"],
        "diagram_interchange_id"
    );
}

#[test]
fn bpmn_linter_reports_invalid_di_label_style_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-di-label-style-reference.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_reference");
    assert_eq!(issue.evidence["invalid_reference_count"], 1);
    assert_eq!(
        issue.evidence["invalid_references"][0]["diagram_id"],
        "Diagram_InvalidDiLabelStyleReference"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["plane_id"],
        "Plane_InvalidDiLabelStyleReference"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["element"],
        "BPMNLabel"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["element_id"],
        "Label_Start"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["attribute"],
        "labelStyle"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["reference"],
        "Missing_LabelStyle"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["expected_scope"],
        "diagram_interchange_id"
    );
}

#[test]
fn bpmn_linter_reports_invalid_di_choreography_shape_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-di-choreography-shape-reference.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_di_reference");
    assert_eq!(issue.evidence["invalid_reference_count"], 1);
    assert_eq!(
        issue.evidence["invalid_references"][0]["diagram_id"],
        "Diagram_InvalidDiChoreographyShapeReference"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["plane_id"],
        "Plane_InvalidDiChoreographyShapeReference"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["element"],
        "BPMNShape"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["element_id"],
        "Shape_Start"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["attribute"],
        "choreographyActivityShape"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["reference"],
        "Missing_ChoreographyShape"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["expected_scope"],
        "diagram_interchange_id"
    );
}
