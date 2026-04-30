use super::*;

#[test]
fn run_lint_command_guides_variable_to_variable_gateway_condition_repair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("invalid_variable_comparison_condition.bpmn");
    let bpmn = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_variable_comparison">
  <bpmn:process id="section_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="present_section" implementation="${{environment.services.runAgent}}">
      {present_section_io}
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="more_sections" default="flow_done" />
    <bpmn:serviceTask id="next_section" implementation="${{environment.services.runAgent}}" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="present_section" />
    <bpmn:sequenceFlow id="flow_check" sourceRef="present_section" targetRef="more_sections" />
    <bpmn:sequenceFlow id="flow_more" sourceRef="more_sections" targetRef="next_section">
      <bpmn:conditionExpression>sectionNumber &lt; totalSections</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="more_sections" targetRef="done" />
    <bpmn:sequenceFlow id="flow_next_done" sourceRef="next_section" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        present_section_io = native_service_task_io(
            "present_section",
            "Present one section and output sectionNumber and totalSections.",
            &[],
            &["sectionNumber", "totalSections"],
        )
    );
    write_file(&path, &bpmn);

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should guide variable-to-variable condition repair",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("sectionNumber &lt; totalSections"));
    assert!(output.rendered.contains("variable-to-variable comparison"));
    assert!(output.rendered.contains("hasMoreSections"));
    assert!(output.rendered.contains("sectionsRemaining > 0"));
    assert!(
        output
            .rendered
            .contains("replace_variable_to_variable_comparison_with_boolean_or_literal")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_variable_comparison_condition_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_guides_count_like_boolean_condition_repair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("invalid_questions_remaining_condition.bpmn");
    let bpmn = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_questions_remaining">
  <bpmn:process id="question_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="make_question" implementation="${{environment.services.runAgent}}">
      {make_question_io}
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    <bpmn:userTask id="ask" />
    <bpmn:userTask id="draft_section" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="make_question" />
    <bpmn:sequenceFlow id="flow_gateway" sourceRef="make_question" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_more" sourceRef="decision" targetRef="ask">
      <bpmn:conditionExpression>questionsRemaining</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_section" sourceRef="decision" targetRef="draft_section">
      <bpmn:conditionExpression>sectionsRemaining</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_ask_done" sourceRef="ask" targetRef="done" />
    <bpmn:sequenceFlow id="flow_section_done" sourceRef="draft_section" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        make_question_io = native_service_task_io(
            "make_question",
            "Return JSON with currentQuestion, questionsRemaining, and sectionsRemaining.",
            &[],
            &["currentQuestion", "questionsRemaining", "sectionsRemaining"],
        )
    );
    write_file(&path, &bpmn);

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render count-like condition compact diagnostic",
    );

    assert_llm_repair_snapshot_shape(
        &output.rendered,
        &[
            "questionsRemaining > 0",
            "sectionsRemaining > 0",
            "Issues: 2",
            "JSON number count",
            "routing count-like variables through bare boolean paths",
            "disambiguate_count_like_boolean_condition",
        ],
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_count_like_boolean_condition_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_snapshots_loop_progress_xml_fix() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("interaction_loop_missing_feedback.bpmn");
    let bpmn = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_interaction_loop">
  <bpmn:process id="interaction_loop" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_question" implementation="${{environment.services.runAgent}}">
      {prepare_question_io}
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="more_questions" default="flow_done" />
    <bpmn:userTask id="ask_user">
      {ask_user_io}
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_question" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="prepare_question" targetRef="more_questions" />
    <bpmn:sequenceFlow id="flow_ask" sourceRef="more_questions" targetRef="ask_user">
      <bpmn:conditionExpression>hasMoreQuestions</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="more_questions" targetRef="done" />
    <bpmn:sequenceFlow id="flow_repeat" sourceRef="ask_user" targetRef="prepare_question" />
  </bpmn:process>
</bpmn:definitions>"#,
        prepare_question_io = native_service_task_io(
            "prepare_question",
            "Return JSON with the next currentQuestion, currentChoices, and hasMoreQuestions.",
            &[],
            &["currentQuestion", "currentChoices", "hasMoreQuestions"],
        ),
        ask_user_io = native_user_dynamic_choice_io(
            "ask_user",
            "Ask the current question.",
            "currentQuestion",
            "currentChoices",
            None,
            "answer",
        )
    );
    write_file(&path, &bpmn);

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render loop progress XML fix",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.loop_risk.unbounded_control_cycle")
    );
    assert!(
        output
            .rendered
            .contains("feed answer into prepare_question")
    );
    assert!(output.rendered.contains("unconditional default exit"));
    assert!(output.rendered.contains("    |Contract: native BPMN loop progress requires in-cycle tasks to consume user feedback and emit the gateway route state through standard IO metadata."));
    assert!(output.rendered.contains("Proposed patch:"));
    assert!(output.rendered.contains("name=\"answer\""));
    assert!(output.rendered.contains("Return unified diff only."));
    assert!(!output.rendered.contains("\nAction:"));
    assert!(!output.rendered.contains("\nFix:"));
    assert!(!output.rendered.contains("\nStructured repair:"));

    insta::assert_snapshot!(
        "qianji_lint_compact_loop_progress_xml_fix",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_guides_duplicate_unconditional_branch_repair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("duplicate_unconditional_branch.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_duplicate_branch">
  <bpmn:process id="approval_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="review" />
    <bpmn:exclusiveGateway id="approval" default="flow_rework" />
    <bpmn:serviceTask id="more" implementation="${environment.services.runAgent}" />
    <bpmn:serviceTask id="rework" implementation="${environment.services.runAgent}" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="review" />
    <bpmn:sequenceFlow id="flow_to_gateway" sourceRef="review" targetRef="approval" />
    <bpmn:sequenceFlow id="flow_approved" sourceRef="approval" targetRef="more">
      <bpmn:conditionExpression>approved</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_rework" sourceRef="approval" targetRef="rework" />
    <bpmn:sequenceFlow id="flow_duplicate" sourceRef="approval" targetRef="more" />
    <bpmn:sequenceFlow id="flow_more_done" sourceRef="more" targetRef="done" />
    <bpmn:sequenceFlow id="flow_rework_done" sourceRef="rework" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should guide duplicate unconditional branch repair",
    );

    for expected_fragment in [
        "Action:",
        "Fix:",
        "Patch focus:",
        "Structured repair:",
        "flow_duplicate",
        "flow_approved",
        "same sourceRef/targetRef",
        "remove_duplicate_unconditional_flow",
        "remove_duplicate_unconditional_gateway_branch",
    ] {
        assert!(
            output.rendered.contains(expected_fragment),
            "compact diagnostic should include {expected_fragment}"
        );
    }

    insta::assert_snapshot!(
        "qianji_lint_compact_duplicate_unconditional_branch_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}
