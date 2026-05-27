use crate::lint::{LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_accepts_complex_gateway_metadata_surface() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-unsupported-gateway.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "complex gateway should lint as standard metadata: {report:?}"
    );
    assert!(report.issues.is_empty());
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
fn bpmn_linter_accepts_event_based_gateway_conditional_wait_target() {
    let report = lint_bpmn_source(&bpmn_fixture_source("event-based-gateway-conditional.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
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
    let Some(source_diagnostic) = issue.source_diagnostic.as_ref() else {
        panic!("unsupported condition should carry a source diagnostic");
    };
    assert!(source_diagnostic.span.start < source_diagnostic.span.end);
    assert!(source_diagnostic.label.contains("bounded native subset"));
    assert_lint_json_snapshot("bpmn_gateway_condition_expression_lint_report", &report);
}
