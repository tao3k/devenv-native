use super::super::{LintDomain, lint_bpmn_source};
use qianji_bpmn_engine::BpmnSourceFile;

#[test]
fn bpmn_linter_rejects_undeclared_qianji_result_output() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "undeclared-qianji-result-output.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Question" sourceRef="Start" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Choose an action.</qianji:prompt>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>shouldEscalate</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question>What should happen?</qianji:question>
            <qianji:choice value="escalate" label="Escalate"/>
            <qianji:result output="blockerAction"/>
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
    assert_eq!(issue.code, "bpmn.undeclared_qianji_interaction_result");
    assert!(issue.summary.contains("blockerAction"));
    assert!(issue.llm_fix_prompt.contains("qianji:outputs"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Add 'blockerAction'"))
    );
}
#[test]
fn bpmn_linter_rejects_choice_input_without_static_or_dynamic_choices() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "missing-qianji-choices.bpmn",
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
          <qianji:outputs>answer</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question ref="currentQuestion"/>
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
    assert_eq!(issue.code, "bpmn.missing_qianji_interaction_choices");
    assert!(issue.llm_fix_prompt.contains("qianji:choices"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("currentChoices"))
    );
}
