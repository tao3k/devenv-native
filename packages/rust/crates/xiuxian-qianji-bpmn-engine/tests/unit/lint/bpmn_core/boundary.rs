use super::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

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
fn bpmn_linter_accepts_non_interrupting_boundary_timer_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source("boundary-timer-non-interrupt.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_boundary_timer_without_expression_as_metadata() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "metadata-boundary-timer-missing-expression.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "empty boundary timer definitions from standard modeler palettes should lint as metadata: {report:?}"
    );
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_receive_task_boundary_timer_as_metadata() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "metadata-receive-task-boundary-timer.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "standard task-family boundary timers should include receiveTask owners: {report:?}"
    );
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_non_interrupting_boundary_message_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source("boundary-message-non-interrupt.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_non_interrupting_boundary_signal_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source("boundary-signal-non-interrupt.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_parallel_multi_instance_non_interrupting_boundary_timer_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "boundary-timer-non-interrupt-parallel-mi.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_sequential_multi_instance_non_interrupting_boundary_timer_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "boundary-timer-non-interrupt-sequential-mi.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_interrupting_boundary_message_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source("boundary-message-interrupt.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_cancel_boundary_on_task_as_metadata() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-cancel-boundary-task.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "cancel boundary on a non-transaction owner should lint as metadata-only: {report:?}"
    );
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_interrupting_conditional_boundary_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source("boundary-conditional-interrupt.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_non_interrupting_conditional_boundary_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "boundary-conditional-non-interrupt.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_standard_loop_non_interrupting_boundary_timer_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "boundary-timer-non-interrupt-standard-loop.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_top_level_error_end_subset() {
    let report = lint_bpmn_source(&bpmn_fixture_source("top-level-error-end.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_reports_escalation_deferred_non_interrupting_boundary_with_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-boundary-escalation-non-interrupt.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_escalation_task_boundary_as_metadata() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-boundary-escalation-task-owner.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}
