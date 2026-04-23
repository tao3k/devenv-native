use super::{bpmn_fixture_source, dmn_fixture_source, lint_bpmn_source, lint_dmn_source};

#[test]
fn linter_reports_ok_for_valid_bpmn_and_dmn_sources() {
    let bpmn_report = lint_bpmn_source(&bpmn_fixture_source("linear-service-task.bpmn"));
    let dmn_report = lint_dmn_source(&dmn_fixture_source("simple-unique-eligibility.dmn"));

    assert!(bpmn_report.ok);
    assert!(bpmn_report.issues.is_empty());
    assert!(dmn_report.ok);
    assert!(dmn_report.issues.is_empty());
}
