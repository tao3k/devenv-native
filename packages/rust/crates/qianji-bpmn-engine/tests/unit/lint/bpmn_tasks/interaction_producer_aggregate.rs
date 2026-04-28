use super::super::{LintDomain, lint_bpmn_source};
use qianji_bpmn_engine::BpmnSourceFile;

#[test]
fn bpmn_linter_aggregates_unbound_dynamic_choice_producers() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "dynamic-qianji-choices-unbound-inputs-multiple.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_First" sourceRef="Start" targetRef="Task_PrepareFirst"/>
    <serviceTask id="Task_PrepareFirst" name="Prepare first" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare the first question.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>contactInfo</qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_First_Ask" sourceRef="Task_PrepareFirst" targetRef="Task_AskFirst"/>
    <userTask id="Task_AskFirst" name="Ask first">
      <extensionElements>
        <qianji:config>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>firstAnswer</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question ref="currentQuestion"/>
            <qianji:choices ref="currentChoices"/>
            <qianji:result output="firstAnswer"/>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
    <sequenceFlow id="Flow_First_Second" sourceRef="Task_AskFirst" targetRef="Task_PrepareSecond"/>
    <serviceTask id="Task_PrepareSecond" name="Prepare second" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare the second question.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>firstAnswer</qianji:inputs>
          <qianji:outputs>nextQuestion,nextChoices</qianji:outputs>
          <qianji:outputSchema name="nextChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Second_Ask" sourceRef="Task_PrepareSecond" targetRef="Task_AskSecond"/>
    <userTask id="Task_AskSecond" name="Ask second">
      <extensionElements>
        <qianji:config>
          <qianji:inputs>nextQuestion,nextChoices</qianji:inputs>
          <qianji:outputs>secondAnswer</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question ref="nextQuestion"/>
            <qianji:choices ref="nextChoices"/>
            <qianji:result output="secondAnswer"/>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
    <sequenceFlow id="Flow_Second_End" sourceRef="Task_AskSecond" targetRef="End"/>
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
        "bpmn.dynamic_qianji_interaction_producer_unbound_inputs"
    );
    assert!(issue.summary.contains("2 matching unbound producers"));
    let findings = issue
        .evidence
        .get("findings")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("aggregate issue should expose findings"));
    assert_eq!(findings.len(), 2);
    let expected_xml = issue
        .structured_repair
        .as_ref()
        .and_then(|repair| repair.get("expected_xml"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("aggregate issue should expose expected XML"));
    assert!(expected_xml.contains("Task_PrepareFirst -> Task_AskFirst"));
    assert!(expected_xml.contains("Task_PrepareSecond -> Task_AskSecond"));
}
