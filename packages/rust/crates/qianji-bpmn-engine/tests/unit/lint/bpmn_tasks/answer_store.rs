use super::super::{LintDomain, lint_bpmn_source};
use qianji_bpmn_engine::BpmnSourceFile;

#[test]
fn bpmn_linter_rejects_redundant_user_answer_store_service_task() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "redundant-user-answer-store.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_Intake" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Contact" sourceRef="Start" targetRef="Task_CollectContact"/>
    <userTask id="Task_CollectContact" name="Collect Contact">
      <extensionElements>
        <qianji:config>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>contactResult</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question>What is your preferred contact method?</qianji:question>
            <qianji:choice value="phone" label="Phone">Call me.</qianji:choice>
            <qianji:choice value="portal" label="Portal">Use secure messaging.</qianji:choice>
            <qianji:result output="contactResult"/>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
    <sequenceFlow id="Flow_Contact_Store" sourceRef="Task_CollectContact" targetRef="Task_StoreContact"/>
    <serviceTask id="Task_StoreContact" name="Store Contact" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Process the contact result and store it as structured contactInfo.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>contactResult</qianji:inputs>
          <qianji:outputs>contactInfo</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Store_Next" sourceRef="Task_StoreContact" targetRef="Task_CollectVisit"/>
    <userTask id="Task_CollectVisit" name="Collect Visit">
      <extensionElements>
        <qianji:config>
          <qianji:tools></qianji:tools>
          <qianji:inputs>contactInfo</qianji:inputs>
          <qianji:outputs>visitResult</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question>What is the reason for your visit?</qianji:question>
            <qianji:choice value="new_condition" label="New condition">First time for this issue.</qianji:choice>
            <qianji:choice value="follow_up" label="Follow-up">Ongoing care.</qianji:choice>
            <qianji:result output="visitResult"/>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
    <sequenceFlow id="Flow_Visit_End" sourceRef="Task_CollectVisit" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.redundant_user_answer_store_service_task");
    assert!(issue.summary.contains("Task_StoreContact"));
    assert!(issue.summary.contains("contactResult"));
    assert!(issue.summary.contains("contactInfo"));
    assert!(issue.source_diagnostic.is_some());
    assert_eq!(
        issue
            .structured_repair
            .as_ref()
            .and_then(|repair| repair.get("strategy"))
            .and_then(serde_json::Value::as_str),
        Some("remove_redundant_user_answer_store_service_tasks")
    );
}
#[test]
fn bpmn_linter_aggregates_redundant_user_answer_store_service_tasks() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "redundant-user-answer-store-multiple.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_Intake" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_One" sourceRef="Start" targetRef="Task_One"/>
    <userTask id="Task_One">
      <extensionElements>
        <qianji:config>
          <qianji:outputs>firstAnswer</qianji:outputs>
          <qianji:interaction type="input">
            <qianji:question>First answer?</qianji:question>
            <qianji:result output="firstAnswer"/>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
    <sequenceFlow id="Flow_One_Store" sourceRef="Task_One" targetRef="Task_StoreOne"/>
    <serviceTask id="Task_StoreOne" name="Store One">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Store the first answer.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>firstAnswer</qianji:inputs>
          <qianji:outputs>storedFirstAnswer</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Store_Second" sourceRef="Task_StoreOne" targetRef="Task_Two"/>
    <userTask id="Task_Two">
      <extensionElements>
        <qianji:config>
          <qianji:inputs>storedFirstAnswer</qianji:inputs>
          <qianji:outputs>secondAnswer</qianji:outputs>
          <qianji:interaction type="input">
            <qianji:question>Second answer?</qianji:question>
            <qianji:result output="secondAnswer"/>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
    <sequenceFlow id="Flow_Two_Store" sourceRef="Task_Two" targetRef="Task_StoreTwo"/>
    <serviceTask id="Task_StoreTwo" name="Store Two">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Store the second answer.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>secondAnswer</qianji:inputs>
          <qianji:outputs>storedSecondAnswer</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Store_End" sourceRef="Task_StoreTwo" targetRef="Task_Review"/>
    <serviceTask id="Task_Review">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Review the answers.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>storedFirstAnswer,storedSecondAnswer</qianji:inputs>
          <qianji:outputs>summary</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Review_End" sourceRef="Task_Review" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.redundant_user_answer_store_service_task");
    assert!(issue.summary.contains("2 matching store serviceTasks"));
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
    assert!(expected_xml.contains("remove Task_StoreOne"));
    assert!(expected_xml.contains("remove Task_StoreTwo"));
    assert!(expected_xml.contains("replace qianji:inputs `storedFirstAnswer` with `firstAnswer`"));
}
