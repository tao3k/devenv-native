use crate::lint::{LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_unsupported_gateway_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-unsupported-gateway.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_complex_gateway");
    assert!(issue.why_it_failed.contains("activation"));
    assert!(issue.llm_fix_prompt.contains("exclusiveGateway"));
    assert!(issue.llm_fix_prompt.contains("parallelGateway"));
    assert_eq!(issue.evidence["element"], "complexGateway");
    assert_eq!(
        issue
            .structured_repair
            .as_ref()
            .and_then(|repair| { repair.get("contract").and_then(serde_json::Value::as_str) }),
        Some("bpmn.native.gateway.complex_deferred.v1")
    );
    assert_lint_json_snapshot("bpmn_unsupported_gateway_lint_report", &report);
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
