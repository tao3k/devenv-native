use super::{LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_missing_compensation_handler_marker_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-transaction-compensation-missing-marker.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_compensation_configuration");
    assert!(issue.summary.contains("tx_refund"));
    assert!(issue.why_it_failed.contains("transaction shell"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("isForCompensation"))
    );
    assert_lint_json_snapshot("bpmn_compensation_missing_marker_lint_report", &report);
}

#[test]
fn bpmn_linter_reports_throw_compensation_end_event_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-throw-compensation-end.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_compensation_configuration");
    assert!(issue.summary.contains("throw_end"));
    assert!(issue.title.contains("Throw compensation end events"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("ordinary `<bpmn:endEvent>`"))
    );
    assert!(issue.llm_fix_prompt.contains("boundary-to-handler"));
    assert_lint_json_snapshot("bpmn_throw_compensation_end_lint_report", &report);
}

#[test]
fn bpmn_linter_reports_async_throw_compensation_end_event_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-throw-compensation-end-async.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_compensation_configuration");
    assert!(issue.summary.contains("tx_throw_end"));
    assert!(
        issue
            .title
            .contains("Asynchronous throw compensation end events")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("waitForCompletion"))
    );
    assert!(issue.llm_fix_prompt.contains("synchronous"));
}

#[test]
fn bpmn_linter_reports_default_compensation_end_event_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-default-compensation-end.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_compensation_configuration");
    assert!(issue.summary.contains("throw_end"));
    assert!(issue.title.contains("Default compensation end events"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("ordinary `<bpmn:endEvent>`"))
    );
    assert!(issue.llm_fix_prompt.contains("default compensation"));
    assert_lint_json_snapshot("bpmn_default_compensation_end_lint_report", &report);
}

#[test]
fn bpmn_linter_reports_throw_compensation_intermediate_event_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-throw-compensation-intermediate.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_compensation_configuration");
    assert!(issue.summary.contains("throw_intermediate"));
    assert!(
        issue
            .title
            .contains("Throw compensation intermediate events")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("normal sequence-flow routing"))
    );
    assert!(issue.llm_fix_prompt.contains("intermediateThrowEvent"));
    assert_lint_json_snapshot("bpmn_throw_compensation_intermediate_lint_report", &report);
}

#[test]
fn bpmn_linter_reports_default_compensation_intermediate_event_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-default-compensation-intermediate.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_compensation_configuration");
    assert!(issue.summary.contains("throw_intermediate"));
    assert!(
        issue
            .title
            .contains("Default compensation intermediate events")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("normal sequence-flow routing"))
    );
    assert!(issue.llm_fix_prompt.contains("default compensation"));
    assert_lint_json_snapshot(
        "bpmn_default_compensation_intermediate_lint_report",
        &report,
    );
}
