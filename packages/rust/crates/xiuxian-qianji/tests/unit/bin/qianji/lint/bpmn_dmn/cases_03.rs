use super::{
    LintCliCommand, TempDir, assert_llm_repair_snapshot_shape, must_ok, run_lint_command,
    stable_temp_output, write_file,
};

#[test]
fn run_lint_command_surfaces_invalid_default_with_task_route_error() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("task_route_with_invalid_default.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_task_and_default">
  <bpmn:process id="task_and_default" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_next" implementation="${environment.services.runAgent}" />
    <bpmn:exclusiveGateway id="decision" default="flow_missing" />
    <bpmn:endEvent id="done" />
    <bpmn:endEvent id="fallback" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_next" />
    <bpmn:sequenceFlow id="flow_yes" sourceRef="decision" targetRef="done">
      <bpmn:conditionExpression>approved</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_fallback" sourceRef="decision" targetRef="fallback" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should surface invalid defaults alongside task route errors",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("Issues: 2"));
    assert!(
        output
            .rendered
            .contains("repair_task_single_outgoing_route")
    );
    assert!(output.rendered.contains("flow_missing"));
    assert!(
        output
            .rendered
            .contains("retarget_default_flow_to_existing_outgoing")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_task_route_plus_invalid_default_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_guides_invalid_default_flow_to_existing_outgoing() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_default_flow.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_default">
  <bpmn:process id="default_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:exclusiveGateway id="decision" default="flow_missing" />
    <bpmn:endEvent id="approved_end" />
    <bpmn:endEvent id="fallback_end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_yes" sourceRef="decision" targetRef="approved_end">
      <bpmn:conditionExpression>approved</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_fallback" sourceRef="decision" targetRef="fallback_end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render invalid default flow compact diagnostic",
    );

    assert_eq!(output.exit_code, 2);
    assert_llm_repair_snapshot_shape(
        &output.rendered,
        &[
            "retarget stale default flow `flow_missing`",
            "Valid outgoing flow ids from gateway `decision`: flow_yes, flow_fallback",
            "default=\"flow_fallback\"",
            "retarget_default_flow_to_existing_outgoing",
            "valid_outgoing_flow_ids:",
        ],
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_invalid_default_flow_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_prefers_stale_renamed_default_branch() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("stale_renamed_default_flow.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_default">
  <bpmn:process id="default_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:exclusiveGateway id="decision" default="Flow_NoVisual" />
    <bpmn:endEvent id="visual_end" />
    <bpmn:endEvent id="fallback_end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="Flow_VisualYes" sourceRef="decision" targetRef="visual_end">
      <bpmn:conditionExpression>involvesVisualQuestions</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="Flow_NoVisual_Join" sourceRef="decision" targetRef="fallback_end">
      <bpmn:conditionExpression>not involvesVisualQuestions</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should prefer the stale renamed default branch",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("default=\"Flow_NoVisual_Join\""));
    assert!(
        output
            .rendered
            .contains("Remove that sequenceFlow conditionExpression")
    );
    assert!(
        output
            .rendered
            .contains("preferred_default_has_condition: true")
    );
    assert!(
        output
            .rendered
            .contains("remove conditionExpression from the selected default")
    );
}

#[test]
fn run_lint_command_guides_string_choice_gateway_condition_repair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_string_choice_condition.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_condition">
  <bpmn:process id="condition_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:exclusiveGateway id="decision" default="flow_default" />
    <bpmn:endEvent id="merge_end" />
    <bpmn:endEvent id="fallback_end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_merge" sourceRef="decision" targetRef="merge_end">
      <bpmn:conditionExpression>chosenOption == 'merge'</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_default" sourceRef="decision" targetRef="fallback_end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render string choice repair guidance",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("chosenOption == 'merge'"));
    assert!(output.rendered.contains("user choice or enum string"));
    assert!(output.rendered.contains("selectedMerge"));
    assert!(
        output
            .rendered
            .contains("replace_string_or_enum_equality_with_boolean_route_variable")
    );
    assert!(output.rendered.contains("native BPMN output"));
}
