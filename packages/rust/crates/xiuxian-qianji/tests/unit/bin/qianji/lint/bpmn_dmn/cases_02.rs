use super::*;

#[test]
fn run_lint_command_guides_user_task_result_output_repair_in_one_patch() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("ambiguous_user_task_outputs.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_user_task_outputs">
  <bpmn:process id="user_task_outputs" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="answer_question">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Ask the user one question.</qianji:prompt>
          <qianji:inputs>currentQuestion,currentChoices</qianji:inputs>
          <qianji:outputs>answer,feedback</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question ref="currentQuestion"/>
            <qianji:choices ref="currentChoices"/>
            <qianji:freeText name="feedback" optional="true"/>
            <qianji:result output="answer"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="answer_question" />
    <bpmn:sequenceFlow id="flow_end" sourceRef="answer_question" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render userTask output repair diagnostic",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.ambiguous_qianji_interaction_outputs")
    );
    assert!(output.rendered.contains("Proposed patch:"));
    assert!(
        output
            .rendered
            .contains("-          <qianji:outputs>answer,feedback</qianji:outputs>")
    );
    assert!(
        output
            .rendered
            .contains("+          <qianji:outputs>answer</qianji:outputs>")
    );
    assert!(output.rendered.contains("Return unified diff only."));

    insta::assert_snapshot!(
        "qianji_lint_compact_user_task_result_output_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_guides_single_outgoing_default_repair_in_one_patch() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_single_outgoing_default.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_default">
  <bpmn:process id="default_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:exclusiveGateway id="decision" default="flow_fallback" />
    <bpmn:serviceTask id="retry" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_retry" sourceRef="decision" targetRef="retry">
      <bpmn:conditionExpression>needsRetry</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="retry" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render complete default branch repair guidance",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("default flow requires branching"));
    assert!(
        output
            .rendered
            .contains("add_or_retarget_unconditional_default_flow")
    );
    assert!(output.rendered.contains("unconditional non-default branch"));
    assert!(
        output
            .rendered
            .contains("default=\"flow_fallback\" needs a real fallback branch")
    );
    assert!(
        output
            .rendered
            .contains("Gateway `decision` currently has 1 outgoing flow(s): flow_retry")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_single_outgoing_default_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_guides_missing_task_outgoing_repair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("missing_task_outgoing.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_missing_task_route">
  <bpmn:process id="missing_task_route" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_next" implementation="${environment.services.runAgent}" />
    <bpmn:exclusiveGateway id="more_questions" default="flow_done" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_next" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="more_questions" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render missing task route compact diagnostic",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.unsupported_task_configuration")
    );
    assert!(output.rendered.contains("task_requires_single_outgoing"));
    assert!(output.rendered.contains("prepare_next"));
    assert!(
        output
            .rendered
            .contains("task must have exactly one outgoing sequenceFlow")
    );
    assert!(
        output
            .rendered
            .contains("Task `prepare_next` currently has 0 outgoing flow(s): none")
    );
    assert!(
        output
            .rendered
            .contains("repair_task_single_outgoing_route")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_missing_task_outgoing_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_surfaces_duplicate_gateway_branch_with_task_route_error() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("task_route_with_duplicate_gateway_branch.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_task_and_gateway">
  <bpmn:process id="task_and_gateway" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_next" implementation="${environment.services.runAgent}" />
    <bpmn:exclusiveGateway id="decision" default="flow_fallback" />
    <bpmn:endEvent id="done" />
    <bpmn:endEvent id="fallback" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_next" />
    <bpmn:sequenceFlow id="flow_yes" sourceRef="decision" targetRef="done">
      <bpmn:conditionExpression>approved</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_duplicate" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_fallback" sourceRef="decision" targetRef="fallback" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should surface source-visible gateway duplicates alongside task route errors",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("Issues: 2"));
    assert!(
        output
            .rendered
            .contains("repair_task_single_outgoing_route")
    );
    assert!(output.rendered.contains("flow_duplicate"));
    assert!(
        output
            .rendered
            .contains("remove_duplicate_unconditional_gateway_branch")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_task_route_plus_duplicate_gateway_branch_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}
