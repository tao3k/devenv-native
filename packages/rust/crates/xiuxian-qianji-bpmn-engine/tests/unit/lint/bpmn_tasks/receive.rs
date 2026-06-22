use crate::lint::{LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_receive_task_multiple_binding_sources_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-receive-task-double-message-binding.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_task_configuration");
    assert!(issue.title.contains("binding"));
    assert!(issue.llm_fix_prompt.contains("messageRef"));
    assert!(issue.llm_fix_prompt.contains("messageEventDefinition"));
    assert_lint_json_snapshot("bpmn_receive_task_double_binding_lint_report", &report);
}
#[test]
fn bpmn_linter_reports_receive_task_non_message_binding_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-receive-task-signal-binding.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_task_configuration");
    assert!(issue.title.contains("unsupported event binding"));
    assert!(issue.llm_fix_prompt.contains("receiveTask"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("signal/timer task events"))
    );
}
