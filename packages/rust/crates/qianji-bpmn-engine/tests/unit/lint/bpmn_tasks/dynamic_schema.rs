use super::super::lint_bpmn_source;
use qianji_bpmn_engine::BpmnSourceFile;

#[test]
fn bpmn_linter_repairs_all_dynamic_choice_ref_producers_without_schema() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "dynamic-qianji-choices-multiple-missing-schema.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_First" sourceRef="Start" targetRef="Task_PrepareFirst"/>
    <serviceTask id="Task_PrepareFirst" name="Prepare first question">
      <extensionElements>
        <qianji:config>
          <qianji:inputs>topic</qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_First_Next" sourceRef="Task_PrepareFirst" targetRef="Task_PrepareNext"/>
    <serviceTask id="Task_PrepareNext" name="Prepare next question">
      <extensionElements>
        <qianji:config>
          <qianji:inputs>answer</qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Next_Question" sourceRef="Task_PrepareNext" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <extensionElements>
        <qianji:config>
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

    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    let Some(line_fixes) = issue
        .structured_repair
        .as_ref()
        .and_then(|repair| repair.get("line_fixes"))
        .and_then(serde_json::Value::as_array)
    else {
        panic!("dynamic choices repair should include line fixes");
    };
    assert_eq!(line_fixes.len(), 2);
    let targets = line_fixes
        .iter()
        .filter_map(|fix| fix.get("target"))
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        targets
            .iter()
            .any(|target| target.contains("Task_PrepareFirst")),
        "expected first producer target in {targets:?}"
    );
    assert!(
        targets
            .iter()
            .any(|target| target.contains("Task_PrepareNext")),
        "expected next producer target in {targets:?}"
    );
}
