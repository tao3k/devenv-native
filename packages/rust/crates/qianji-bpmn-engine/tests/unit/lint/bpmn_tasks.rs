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
fn bpmn_linter_reports_native_user_task_rendering_as_deferred() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "native-user-task-rendering.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Review" sourceRef="Start" targetRef="Task_Review"/>
    <userTask id="Task_Review" name="Review request">
      <rendering id="Rendering_Form"/>
    </userTask>
    <sequenceFlow id="Flow_Review_End" sourceRef="Task_Review" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_human_task_rendering");
    assert!(issue.summary.contains("Task_Review"));
    assert!(issue.why_it_failed.contains("qianji:interaction"));
    assert!(issue.llm_fix_prompt.contains("typed `qianji:interaction`"));
    assert_eq!(issue.evidence["task_id"], "Task_Review");
    assert_eq!(issue.evidence["element"], "rendering");
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_reports_resource_parameter_binding_assignment_semantics() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "human-task-resource-parameter-binding.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Review" sourceRef="Start" targetRef="Task_Review"/>
    <userTask id="Task_Review" name="Review request">
      <potentialOwner name="review_team">
        <resourceRef>reviewers</resourceRef>
        <resourceParameterBinding parameterRef="region">
          <formalExpression>emea</formalExpression>
        </resourceParameterBinding>
      </potentialOwner>
    </userTask>
    <sequenceFlow id="Flow_Review_End" sourceRef="Task_Review" targetRef="End"/>
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
        "bpmn.unsupported_human_task_assignment_semantics"
    );
    assert!(issue.summary.contains("resourceParameterBinding"));
    assert!(issue.why_it_failed.contains("routing metadata only"));
    assert!(issue.llm_fix_prompt.contains("potentialOwner"));
    assert_eq!(issue.evidence["task_id"], "Task_Review");
    assert_eq!(issue.evidence["element"], "resourceParameterBinding");
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_reports_generic_performer_assignment_semantics() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "human-task-generic-performer.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Review" sourceRef="Start" targetRef="Task_Review"/>
    <userTask id="Task_Review" name="Review request">
      <performer name="reviewer"/>
    </userTask>
    <sequenceFlow id="Flow_Review_End" sourceRef="Task_Review" targetRef="End"/>
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
        "bpmn.unsupported_human_task_assignment_semantics"
    );
    assert!(issue.summary.contains("<performer>"));
    assert_eq!(issue.evidence["task_id"], "Task_Review");
    assert_eq!(issue.evidence["element"], "performer");
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
    <sequenceFlow id="Flow_Start_Prepare" sourceRef="Start" targetRef="Task_Prepare"/>
    <serviceTask id="Task_Prepare" name="Prepare question" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare currentQuestion and currentChoices.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Prepare_Question" sourceRef="Task_Prepare" targetRef="Task_Question"/>
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

#[test]
fn bpmn_linter_accepts_dynamic_qianji_choice_ref() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "dynamic-qianji-choices.bpmn",
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
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

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
