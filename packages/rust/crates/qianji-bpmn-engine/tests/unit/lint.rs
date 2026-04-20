use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnSourceFile, DmnSourceFile, LintDomain, lint_bpmn_source, lint_dmn_source,
};

#[test]
fn bpmn_linter_reports_unsupported_inclusive_gateway_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-unsupported-gateway.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    assert_lint_json_snapshot("bpmn_unsupported_gateway_lint_report", &report);
}

#[test]
fn dmn_linter_reports_multiple_decisions_with_llm_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("invalid-multiple-decisions.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    assert_lint_json_snapshot("dmn_multiple_decisions_lint_report", &report);
}

#[test]
fn dmn_linter_reports_unsupported_unary_test_with_llm_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("invalid-unsupported-unary-test.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_unary_test");
    assert!(issue.summary.contains("time(\"09:00:00\")"));
    assert!(issue.why_it_failed.contains("date(\"YYYY-MM-DD\")"));
    assert!(issue.why_it_failed.contains("ISO date ranges"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("date(\"2026-01-01\") <= ? < date(\"2026-01-31\")")
    );
}

#[test]
fn linter_reports_ok_for_valid_bpmn_and_dmn_sources() {
    let bpmn_report = lint_bpmn_source(&bpmn_fixture_source("linear-service-task.bpmn"));
    let dmn_report = lint_dmn_source(&dmn_fixture_source("simple-unique-eligibility.dmn"));

    assert!(bpmn_report.ok);
    assert!(bpmn_report.issues.is_empty());
    assert!(dmn_report.ok);
    assert!(dmn_report.issues.is_empty());
}

#[test]
fn bpmn_linter_reports_missing_intermediate_event_definition_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-intermediate-catch-missing-event-definition.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.missing_required_node_element");
    assert!(issue.summary.contains("wait_missing"));
    assert!(issue.llm_fix_prompt.contains("event_definition"));
}

#[test]
fn bpmn_linter_reports_non_interrupting_boundary_timer_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-non-interrupting-boundary-timer.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_boundary_configuration");
    assert!(issue.summary.contains("review_timeout"));
    assert!(issue.llm_fix_prompt.contains("cancelActivity=\"true\""));
}

#[test]
fn bpmn_linter_reports_missing_called_process_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-call-activity-missing-target.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unknown_called_process");
    assert!(issue.summary.contains("missing_process"));
    assert!(issue.llm_fix_prompt.contains("calledElement"));
}

#[test]
fn bpmn_linter_reports_invalid_event_based_gateway_target_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-event-based-gateway-task-target.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "bpmn.unsupported_event_based_gateway_configuration"
    );
    assert!(issue.summary.contains("wait_race"));
    assert!(issue.llm_fix_prompt.contains("eventBasedGateway"));
}

#[test]
fn bpmn_linter_reports_invalid_standard_loop_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-standard-loop-missing-limit.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    assert_lint_json_snapshot("bpmn_standard_loop_missing_limit_lint_report", &report);
}

#[test]
fn bpmn_linter_reports_parallel_multi_instance_deferred_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-multi-instance-deferred.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_loop_configuration");
    assert!(issue.summary.contains("review"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("multiInstanceLoopCharacteristics")
    );
}

#[test]
fn bpmn_linter_reports_missing_multi_instance_cardinality_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-sequential-multi-instance-missing-cardinality.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_loop_configuration");
    assert!(issue.summary.contains("review"));
    assert!(issue.llm_fix_prompt.contains("loopCardinality"));
}

fn bpmn_fixture_source(name: &str) -> BpmnSourceFile {
    let path = format!("{}/tests/fixtures/bpmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    BpmnSourceFile::new(name, contents)
}

fn dmn_fixture_source(name: &str) -> DmnSourceFile {
    let path = format!("{}/tests/fixtures/dmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    DmnSourceFile::new(name, contents)
}

fn assert_lint_json_snapshot(name: &str, value: impl serde::Serialize) {
    insta::with_settings!({
        snapshot_path => "../snapshots",
        prepend_module_to_snapshot => false,
        sort_maps => true,
    }, {
        insta::assert_json_snapshot!(name, value);
    });
}
