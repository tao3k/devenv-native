use super::{
    LintCliCommand, TempDir, assert_llm_repair_snapshot_shape, must_ok, native_user_choice_io,
    run_lint_command, stable_temp_output, write_file,
};

#[test]
fn run_lint_command_guides_non_boolean_interaction_choice_condition_repair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("non_boolean_interaction_choice_condition.bpmn");
    let bpmn = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_interaction_gateway">
  <bpmn:process id="interaction_gateway_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="ask_question">
      {ask_question_io}
    </bpmn:userTask>
    <bpmn:exclusiveGateway id="has_more" default="flow_done" />
    <bpmn:endEvent id="repeat" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="ask_question" />
    <bpmn:sequenceFlow id="flow_answer" sourceRef="ask_question" targetRef="has_more" />
    <bpmn:sequenceFlow id="flow_more" sourceRef="has_more" targetRef="repeat">
      <bpmn:conditionExpression>moreQuestionsNeeded</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="has_more" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        ask_question_io = native_user_choice_io(
            "ask_question",
            "Ask the user whether more clarification is needed.",
            "choice",
            r#"[{"value":"need_more_clarification","label":"Ask another question"},{"value":"ready_to_proceed","label":"Proceed"}]"#,
            "moreQuestionsNeeded",
        )
    );
    write_file(&path, &bpmn);

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should guide non-boolean interaction choice repair",
    );

    assert_llm_repair_snapshot_shape(
        &output.rendered,
        &[
            "bpmn.non_boolean_interaction_choice_condition",
            "need_more_clarification",
            "ready_to_proceed",
            "align_interaction_choice_output_with_boolean_gateway",
            "needsMoreQuestions",
        ],
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_non_boolean_interaction_choice_condition_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_snapshots_gateway_condition_compact_diagnostic() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_condition.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_condition">
  <bpmn:process id="condition_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:exclusiveGateway id="decision" default="flow_default" />
    <bpmn:endEvent id="approved_end" />
    <bpmn:endEvent id="fallback_end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_yes" sourceRef="decision" targetRef="approved_end">
      <bpmn:conditionExpression>approved == true</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_default" sourceRef="decision" targetRef="fallback_end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render gateway condition compact diagnostic",
    );

    assert_llm_repair_snapshot_shape(
        &output.rendered,
        &[
            "rewrite this condition into the bounded native subset",
            "Allowed forms:",
            "approved == true",
            "rewrite_condition_to_bounded_subset",
        ],
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_gateway_condition_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}
