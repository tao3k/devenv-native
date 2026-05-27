use super::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_multiple_event_definition_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-multiple-event-definition.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_multiple_event_definitions");
    assert!(issue.summary.contains("wait_multiple"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("multiple_event_definition_deferred")
    );
    assert!(issue.why_it_failed.contains("multipleEventDefinition"));
}

#[test]
fn bpmn_linter_reports_parallel_multiple_event_definition_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-parallel-multiple-event-definition.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_multiple_event_definitions");
    assert!(issue.summary.contains("wait_parallel_multiple"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("parallel_multiple_event_definition_deferred")
    );
    assert!(
        issue
            .why_it_failed
            .contains("parallelMultipleEventDefinition")
    );
}

#[test]
fn bpmn_linter_reports_multiple_event_concrete_definitions_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-multiple-event-definitions.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_multiple_event_definitions");
    assert!(issue.summary.contains("wait_multiple"));
    assert!(issue.llm_fix_prompt.contains("multiple_event_definitions"));
    assert!(
        issue
            .why_it_failed
            .contains("one concrete event definition")
    );
}

#[test]
fn bpmn_linter_reports_escalation_deferred_start_event_with_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-escalation-start-event.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_escalation_event");
    assert!(issue.summary.contains("start"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("escalation_start_event_deferred")
    );
    assert!(issue.why_it_failed.contains("event-subprocess"));
}

#[test]
fn bpmn_linter_accepts_link_event_metadata_surface() {
    let report = lint_bpmn_source(&bpmn_fixture_source("link-event-metadata.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "standard link events should lint cleanly: {report:?}"
    );
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_intermediate_timer_without_expression_as_metadata() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-intermediate-timer-event.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "empty timer definitions from standard modeler palettes should lint as metadata: {report:?}"
    );
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_intermediate_throw_event_metadata_surface() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "intermediate-throw-event-metadata.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "standard intermediate throw events should lint cleanly: {report:?}"
    );
    assert!(report.issues.is_empty());
}
