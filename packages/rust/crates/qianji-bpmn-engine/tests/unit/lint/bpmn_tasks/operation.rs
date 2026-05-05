use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};
use qianji_bpmn_engine::{LintIssue, LintReport};

#[test]
fn bpmn_linter_reports_task_operation_binding_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-task-operation-binding.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 3);

    let service_issue = issue_for_task(&report, "invoke_service");
    assert_eq!(service_issue.code, "bpmn.unsupported_operation_binding");
    assert_eq!(service_issue.evidence["task_kind"], "serviceTask");
    assert_eq!(service_issue.evidence["operation_ref"], "Operation_Invoke");
    assert!(
        service_issue
            .why_it_failed
            .contains("interface and operation catalogs as metadata")
    );
    assert!(
        service_issue
            .llm_fix_prompt
            .contains("host-dispatched task metadata")
    );
    assert_eq!(
        service_issue
            .structured_repair
            .as_ref()
            .and_then(|repair| repair["contract"].as_str()),
        Some("bpmn.native.task.operation_binding_deferred.v1")
    );
    assert!(service_issue.source_diagnostic.is_some());

    let send_issue = issue_for_task(&report, "send_update");
    assert_eq!(send_issue.evidence["task_kind"], "sendTask");
    assert_eq!(send_issue.evidence["operation_ref"], "Operation_Send");
    assert_eq!(
        send_issue.evidence["bounded_surface"][3],
        "messageRef_or_messageEventDefinition_for_send_receive_tasks"
    );

    let receive_issue = issue_for_task(&report, "receive_reply");
    assert_eq!(receive_issue.evidence["task_kind"], "receiveTask");
    assert_eq!(receive_issue.evidence["operation_ref"], "Operation_Receive");
}

fn issue_for_task<'a>(report: &'a LintReport, task_id: &str) -> &'a LintIssue {
    report
        .issues
        .iter()
        .find(|issue| issue.evidence["task_id"].as_str() == Some(task_id))
        .unwrap_or_else(|| panic!("expected operation binding issue for task {task_id}"))
}
