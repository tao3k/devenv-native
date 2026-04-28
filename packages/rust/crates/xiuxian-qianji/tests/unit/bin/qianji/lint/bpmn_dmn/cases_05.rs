use super::*;

#[test]
fn run_lint_command_snapshots_dynamic_choices_output_schema_fix() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("dynamic_choices_missing_output_schema.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_dynamic_choices">
  <bpmn:process id="dynamic_choices" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_question" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return JSON with currentQuestion and currentChoices.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>topic</qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:userTask id="ask_user">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Ask the generated question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>answer</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question ref="currentQuestion"/>
            <qianji:choices ref="currentChoices"/>
            <qianji:result output="answer"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_question" />
    <bpmn:sequenceFlow id="flow_ask" sourceRef="prepare_question" targetRef="ask_user" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="ask_user" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render dynamic choices output schema XML fix",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.missing_qianji_dynamic_choices_output_schema")
    );
    assert!(output.rendered.contains("Proposed patch:"));
    assert!(
        output
            .rendered
            .contains("<qianji:outputSchema name=\"currentChoices\" kind=\"choice_array\"")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_dynamic_choices_output_schema_fix",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_snapshots_static_interaction_producer_fix() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("static_interaction_producer.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_static_interaction">
  <bpmn:process id="static_interaction" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_screen" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare a fixed screening question with fixed choices.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:userTask id="ask_screen">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Ask the fixed question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>safetyAnswer</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question ref="currentQuestion"/>
            <qianji:choices ref="currentChoices"/>
            <qianji:result output="safetyAnswer"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_screen" />
    <bpmn:sequenceFlow id="flow_ask" sourceRef="prepare_screen" targetRef="ask_screen" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="ask_screen" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render static interaction producer XML fix",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.static_qianji_interaction_producer")
    );
    assert!(output.rendered.contains("Expected XML:"));
    assert!(
        output
            .rendered
            .contains("<qianji:choice value=\"option_value\"")
    );
    assert!(
        output
            .rendered
            .contains("static UI producer should not invoke an LLM")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_static_interaction_producer_fix",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_snapshots_redundant_static_interaction_producer_fix() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("redundant_static_interaction_producer.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_redundant_static_interaction">
  <bpmn:process id="redundant_static_interaction" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_screen" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare a fixed screening question with fixed choices.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:userTask id="ask_screen">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Ask the fixed question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>safetyAnswer</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question>Are any of these happening now or worsening quickly?</qianji:question>
            <qianji:choice value="routine" label="Routine">Standard appointment needed</qianji:choice>
            <qianji:choice value="urgent" label="Urgent">Need care soon</qianji:choice>
            <qianji:result output="safetyAnswer"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_screen" />
    <bpmn:sequenceFlow id="flow_ask" sourceRef="prepare_screen" targetRef="ask_screen" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="ask_screen" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render redundant static producer fix",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.redundant_static_qianji_interaction_producer")
    );
    assert!(output.rendered.contains("Expected XML:"));
    assert!(
        output
            .rendered
            .contains("Remove serviceTask 'prepare_screen'")
    );
    assert!(
        output
            .rendered
            .contains("redundant static UI producer should be removed")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_redundant_static_interaction_producer_fix",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_snapshots_unbound_dynamic_choices_input_fix() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("unbound_dynamic_choices_input.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_unbound_dynamic_choices">
  <bpmn:process id="unbound_dynamic_choices" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_visit_category" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Prepare question for visit reason category.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>contactInfo</qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:userTask id="ask_visit_category">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Ask the generated question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>visitCategoryResult</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question ref="currentQuestion"/>
            <qianji:choices ref="currentChoices"/>
            <qianji:result output="visitCategoryResult"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_visit_category" />
    <bpmn:sequenceFlow id="flow_ask" sourceRef="prepare_visit_category" targetRef="ask_visit_category" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="ask_visit_category" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render unbound dynamic choices input fix",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.dynamic_qianji_interaction_producer_unbound_inputs")
    );
    assert!(output.rendered.contains("Expected XML:"));
    assert!(output.rendered.contains("contactInfo"));
    assert!(
        output
            .rendered
            .contains("dynamic choices producer must bind qianji:inputs by name")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_unbound_dynamic_choices_input_fix",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}
