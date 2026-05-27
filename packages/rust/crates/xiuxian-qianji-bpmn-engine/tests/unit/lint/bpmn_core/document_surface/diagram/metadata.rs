use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_accepts_standard_diagram_interchange_as_native_surface() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-bpmn-diagram.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "standard BPMNDI should be accepted as native diagram interchange: {report:?}"
    );
    assert!(report.issues.is_empty());
}
