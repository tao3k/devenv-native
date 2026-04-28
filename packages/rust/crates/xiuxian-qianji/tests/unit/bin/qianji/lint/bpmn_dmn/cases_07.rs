use super::*;

#[test]
fn run_lint_command_accepts_complete_service_task_tool_scope() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("complete_tool_scope.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_tool_scope">
  <bpmn:process id="tool_scope" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="run_tests" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Run the exact verification command and return exitCode.</qianji:prompt>
          <qianji:tools>bash</qianji:tools>
          <qianji:toolScope tool="bash" command="npm test" timeoutSeconds="120" writes="false" network="false"/>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>exitCode</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="run_tests" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="run_tests" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should accept complete tool-scope contract",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("no blocking issues found"));
}

#[test]
fn run_lint_command_guides_ambiguous_interaction_choices_ref_repair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("ambiguous_interaction_choices_ref.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_interaction_choices">
  <bpmn:process id="interaction_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="approve_design">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Approve the current design section.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>designSection</qianji:inputs>
          <qianji:outputs>sectionApproved</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question ref="designSection"/>
            <qianji:choices ref="designSection"/>
            <qianji:freeText name="revisionFeedback" optional="true"/>
            <qianji:result output="sectionApproved"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:userTask id="review_spec">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Review the written spec file.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>designDocPath</qianji:inputs>
          <qianji:outputs>userReviewApproved</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question ref="designDocPath"/>
            <qianji:choices ref="designDocPath"/>
            <qianji:freeText name="specChanges" optional="true"/>
            <qianji:result output="userReviewApproved"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="approve_design" />
    <bpmn:sequenceFlow id="flow_review" sourceRef="approve_design" targetRef="review_spec" />
    <bpmn:sequenceFlow id="flow_end" sourceRef="review_spec" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render ambiguous interaction choices-ref diagnostic",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("Issues: 2"));
    assert!(
        output
            .rendered
            .contains("bpmn.ambiguous_qianji_interaction_choices_ref")
    );
    assert!(output.rendered.contains("designSection"));
    assert!(output.rendered.contains("designDocPath"));
    assert!(
        output
            .rendered
            .contains("split_question_text_from_dynamic_choices")
    );
    assert!(output.rendered.contains("currentChoices"));
    assert!(
        output
            .rendered
            .contains("replace_dynamic_choices_with_static_choices")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_ambiguous_interaction_choices_ref_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_guides_non_boolean_interaction_choice_condition_repair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("non_boolean_interaction_choice_condition.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_interaction_gateway">
  <bpmn:process id="interaction_gateway_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="ask_question">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Ask the user whether more clarification is needed.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>questionText</qianji:inputs>
          <qianji:outputs>moreQuestionsNeeded</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question ref="questionText"/>
            <qianji:choice value="need_more_clarification" label="Ask another question"/>
            <qianji:choice value="ready_to_proceed" label="Proceed"/>
            <qianji:freeText name="userAnswer" optional="true"/>
            <qianji:result output="moreQuestionsNeeded"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
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
    );

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
            "rewrite this condition into qianji's bounded subset",
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
