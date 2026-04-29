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
