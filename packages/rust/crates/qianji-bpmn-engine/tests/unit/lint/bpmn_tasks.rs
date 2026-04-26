use super::{LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source};
use qianji_bpmn_engine::BpmnSourceFile;

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

#[test]
fn bpmn_linter_rejects_unsupported_qianji_interaction_type() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-qianji-interaction-type.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Question" sourceRef="Start" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Answer the question.</qianji:prompt>
          <qianji:outputs>answer</qianji:outputs>
          <qianji:interaction type="free_form">
            <qianji:question>What should we build?</qianji:question>
            <qianji:freeText name="answer"/>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
    <sequenceFlow id="Flow_Question_End" sourceRef="Task_Question" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_qianji_interaction_type");
    assert!(issue.summary.contains("free_form"));
    assert!(issue.llm_fix_prompt.contains("input"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("choice_input"))
    );
}
