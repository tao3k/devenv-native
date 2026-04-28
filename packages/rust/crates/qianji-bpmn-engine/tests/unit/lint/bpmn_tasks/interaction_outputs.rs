use super::super::{LintDomain, assert_lint_json_snapshot, lint_bpmn_source};
use qianji_bpmn_engine::BpmnSourceFile;

#[test]
fn bpmn_linter_rejects_user_task_interaction_with_multiple_outputs() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "ambiguous-user-task-outputs.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Question" sourceRef="Start" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Answer the generated question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>answer,feedback</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question ref="currentQuestion"/>
            <qianji:choices ref="currentChoices"/>
            <qianji:freeText name="feedback" optional="true"/>
            <qianji:result output="answer"/>
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
    assert_eq!(issue.code, "bpmn.ambiguous_qianji_interaction_outputs");
    assert!(
        issue
            .llm_fix_prompt
            .contains("<qianji:outputs>answer</qianji:outputs>")
    );
    let Some(line_fixes) = issue
        .structured_repair
        .as_ref()
        .and_then(|repair| repair.get("line_fixes"))
        .and_then(|value| value.as_array())
    else {
        panic!("ambiguous userTask outputs should carry line_fixes");
    };
    assert_eq!(line_fixes.len(), 1);
    assert_lint_json_snapshot("bpmn_user_task_interaction_outputs_lint_report", &report);
}
#[test]
fn bpmn_linter_rejects_dynamic_choice_ref_without_producer_schema() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "dynamic-qianji-choices-missing-schema.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Prepare" sourceRef="Start" targetRef="Task_Prepare"/>
    <serviceTask id="Task_Prepare" name="Prepare question" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare currentQuestion and currentChoices.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>topic</qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Prepare_Question" sourceRef="Task_Prepare" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Answer the generated question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>answer</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question ref="currentQuestion"/>
            <qianji:choices ref="currentChoices"/>
            <qianji:result output="answer"/>
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
    assert_eq!(
        issue.code,
        "bpmn.missing_qianji_dynamic_choices_output_schema"
    );
    assert!(issue.summary.contains("Task_Prepare"));
    assert!(
        issue
            .structured_repair
            .as_ref()
            .is_some_and(|repair| repair.get("line_fixes").is_some())
    );
}
