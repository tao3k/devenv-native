use super::super::{LintDomain, assert_lint_json_snapshot, lint_bpmn_source};
use qianji_bpmn_engine::BpmnSourceFile;

#[test]
fn bpmn_linter_rejects_static_qianji_interaction_producer() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "static-qianji-interaction-producer.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Prepare" sourceRef="Start" targetRef="Task_PrepareScreen"/>
    <serviceTask id="Task_PrepareScreen" name="Prepare fixed question" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare the fixed safety screening question and fixed choices.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Prepare_Question" sourceRef="Task_PrepareScreen" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Answer the fixed question.</qianji:prompt>
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
    assert_eq!(issue.code, "bpmn.static_qianji_interaction_producer");
    assert!(issue.summary.contains("Task_PrepareScreen"));
    assert!(issue.llm_fix_prompt.contains("qianji:choice"));
    assert!(issue.source_diagnostic.is_some());
    assert_eq!(
        issue
            .structured_repair
            .as_ref()
            .and_then(|repair| repair.get("strategy"))
            .and_then(serde_json::Value::as_str),
        Some("inline_static_interaction_on_user_task")
    );
    assert_lint_json_snapshot("bpmn_static_interaction_producer_lint_report", &report);
}
#[test]
fn bpmn_linter_rejects_redundant_static_qianji_interaction_producer() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "redundant-static-qianji-interaction-producer.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Prepare" sourceRef="Start" targetRef="Task_PrepareScreen"/>
    <serviceTask id="Task_PrepareScreen" name="Prepare fixed question" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare the fixed safety screening question and fixed choices.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Prepare_Question" sourceRef="Task_PrepareScreen" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Answer the fixed question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>answer</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question>Are any of these happening now or worsening quickly?</qianji:question>
            <qianji:choice value="routine" label="Routine">Standard appointment needed</qianji:choice>
            <qianji:choice value="urgent" label="Urgent">Need care soon</qianji:choice>
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
        "bpmn.redundant_static_qianji_interaction_producer"
    );
    assert!(issue.summary.contains("Task_PrepareScreen"));
    assert!(issue.source_diagnostic.is_some());
    assert_eq!(
        issue
            .structured_repair
            .as_ref()
            .and_then(|repair| repair.get("strategy"))
            .and_then(serde_json::Value::as_str),
        Some("remove_redundant_static_interaction_producer")
    );
}
#[test]
fn bpmn_linter_rejects_dynamic_choice_producer_with_unbound_inputs() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "dynamic-qianji-choices-unbound-inputs.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Prepare" sourceRef="Start" targetRef="Task_PrepareVisitCategory"/>
    <serviceTask id="Task_PrepareVisitCategory" name="Prepare visit category" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare question for visit reason category.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>contactInfo</qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Prepare_Question" sourceRef="Task_PrepareVisitCategory" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Answer the generated question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>visitCategoryResult</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question ref="currentQuestion"/>
            <qianji:choices ref="currentChoices"/>
            <qianji:result output="visitCategoryResult"/>
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
        "bpmn.dynamic_qianji_interaction_producer_unbound_inputs"
    );
    assert!(issue.summary.contains("Task_PrepareVisitCategory"));
    assert!(issue.summary.contains("contactInfo"));
    assert!(issue.source_diagnostic.is_some());
    assert_eq!(
        issue
            .structured_repair
            .as_ref()
            .and_then(|repair| repair.get("strategy"))
            .and_then(serde_json::Value::as_str),
        Some("bind_dynamic_choices_inputs_or_inline_static_interaction")
    );
}
#[test]
fn bpmn_linter_accepts_dynamic_choice_producer_with_bound_inputs() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "dynamic-qianji-choices-bound-inputs.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Prepare" sourceRef="Start" targetRef="Task_PrepareVisitCategory"/>
    <serviceTask id="Task_PrepareVisitCategory" name="Prepare visit category" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Use contactInfo to prepare a visit reason question and choices.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>contactInfo</qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Prepare_Question" sourceRef="Task_PrepareVisitCategory" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Answer the generated question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>visitCategoryResult</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question ref="currentQuestion"/>
            <qianji:choices ref="currentChoices"/>
            <qianji:result output="visitCategoryResult"/>
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
    assert!(report.ok);
    assert!(report.issues.is_empty());
}
