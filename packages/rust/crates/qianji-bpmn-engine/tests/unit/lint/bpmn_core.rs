use super::{LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_unsupported_gateway_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-unsupported-gateway.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    assert_lint_json_snapshot("bpmn_unsupported_gateway_lint_report", &report);
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
fn bpmn_linter_reports_unsupported_gateway_condition_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-exclusive-gateway-unsupported-condition.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_gateway_configuration");
    assert!(issue.summary.contains("decision"));
    assert!(issue.why_it_failed.contains("numeric comparisons"));
    assert!(issue.llm_fix_prompt.contains("amount > 100"));
    assert_lint_json_snapshot("bpmn_gateway_condition_expression_lint_report", &report);
}
