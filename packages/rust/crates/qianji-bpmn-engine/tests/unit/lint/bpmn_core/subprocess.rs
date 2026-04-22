use super::{LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source};

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
fn bpmn_linter_reports_embedded_subprocess_missing_end_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-embedded-subprocess-missing-end.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_subprocess_configuration");
    assert!(issue.summary.contains("inline_review"));
    assert!(issue.why_it_failed.contains("nested `endEvent`"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("<bpmn:endEvent>"))
    );
    assert!(issue.llm_fix_prompt.contains("embedded `subProcess` body"));
}

#[test]
fn bpmn_linter_reports_event_subprocess_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-compensation-event-subprocess.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_subprocess_configuration");
    assert!(issue.summary.contains("comp_handler"));
    assert!(issue.why_it_failed.contains("event subprocesses"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("triggeredByEvent=\"true\""))
    );
    assert!(issue.llm_fix_prompt.contains("boundary-event"));
    assert_lint_json_snapshot("bpmn_event_subprocess_lint_report", &report);
}

#[test]
fn bpmn_linter_accepts_embedded_subprocess_error_boundary_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "embedded-subprocess-error-boundary.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_embedded_subprocess_interrupting_external_boundary_subset() {
    for fixture_name in [
        "embedded-subprocess-timer-boundary.bpmn",
        "embedded-subprocess-message-boundary.bpmn",
        "embedded-subprocess-signal-boundary.bpmn",
    ] {
        let report = lint_bpmn_source(&bpmn_fixture_source(fixture_name));
        assert_eq!(report.domain, LintDomain::Bpmn);
        assert!(report.ok);
        assert!(report.issues.is_empty());
    }
}

#[test]
fn bpmn_linter_accepts_embedded_subprocess_mixed_boundary_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "embedded-subprocess-mixed-boundaries.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_reports_embedded_subprocess_error_missing_boundary_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-embedded-subprocess-error-missing-boundary.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_subprocess_configuration");
    assert!(issue.summary.contains("inline_review"));
    assert!(issue.why_it_failed.contains("embedded `subProcess` owner"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("<bpmn:errorEventDefinition>"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("embedded `<bpmn:subProcess>` body")
    );
}

#[test]
fn bpmn_linter_accepts_call_activity_error_boundary_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source("call-activity-error-boundary.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_call_activity_interrupting_external_boundary_subset() {
    for fixture_name in [
        "call-activity-timer-boundary.bpmn",
        "call-activity-message-boundary.bpmn",
        "call-activity-signal-boundary.bpmn",
    ] {
        let report = lint_bpmn_source(&bpmn_fixture_source(fixture_name));
        assert_eq!(report.domain, LintDomain::Bpmn);
        assert!(report.ok);
        assert!(report.issues.is_empty());
    }
}

#[test]
fn bpmn_linter_accepts_call_activity_mixed_boundary_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source("call-activity-mixed-boundaries.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_transaction_interrupting_external_boundary_subset() {
    for fixture_name in [
        "transaction-timer-boundary.bpmn",
        "transaction-message-boundary.bpmn",
        "transaction-signal-boundary.bpmn",
    ] {
        let report = lint_bpmn_source(&bpmn_fixture_source(fixture_name));
        assert_eq!(report.domain, LintDomain::Bpmn);
        assert!(report.ok);
        assert!(report.issues.is_empty());
    }
}

#[test]
fn bpmn_linter_accepts_transaction_mixed_boundary_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source("transaction-mixed-boundaries.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_transaction_mixed_cancel_boundary_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "transaction-mixed-cancel-boundaries.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_transaction_mixed_cancel_and_error_boundary_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "transaction-mixed-cancel-error-boundaries.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_reports_call_activity_error_missing_boundary_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-call-activity-error-missing-boundary.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_subprocess_configuration");
    assert!(issue.summary.contains("invoke_review"));
    assert!(issue.why_it_failed.contains("same-package `callActivity`"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("<bpmn:errorEventDefinition>"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("same `<bpmn:callActivity>` owner")
    );
}
