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
          <qianji:inputs></qianji:inputs>
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
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_First_Next" sourceRef="Task_PrepareFirst" targetRef="Task_PrepareNext"/>
    <serviceTask id="Task_PrepareNext" name="Prepare next question">
      <extensionElements>
        <qianji:config>
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
