use super::*;

fn stable_temp_output(output: &str, temp_dir: &TempDir) -> String {
    output.replace(&temp_dir.path().display().to_string(), "$TEMP")
}

fn assert_llm_repair_snapshot_shape(output: &str, expected_fragments: &[&str]) {
    for required_section in [
        "Action:",
        "Fix:",
        "Patch focus:",
        "Examples:",
        "Forbidden forms:",
        "Structured repair:",
        "- strategy:",
        "- contract:",
    ] {
        assert!(
            output.contains(required_section),
            "compact diagnostic should include {required_section}"
        );
    }

    for expected_fragment in expected_fragments {
        assert!(
            output.contains(expected_fragment),
            "compact diagnostic should include {expected_fragment}"
        );
    }
}

#[test]
fn run_lint_command_renders_failure_with_llm_guidance() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_invalid_gateway">
  <bpmn:process id="gateway_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:inclusiveGateway id="decision" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="decision" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render failure output",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.starts_with("[lint:error]"));
    assert!(
        output
            .rendered
            .contains("bpmn.unsupported_gateway_configuration")
    );
    assert!(output.rendered.contains("Fix:"));
    assert!(output.rendered.contains("Structured repair:"));
    assert!(output.rendered.contains("inclusiveGateway"));
}

#[test]
fn run_lint_command_renders_bpmn_json_report() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_invalid_gateway">
  <bpmn:process id="gateway_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:inclusiveGateway id="decision" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="decision" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::BpmnJson { path }),
        "lint command should render JSON output",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "lint JSON should parse",
    );

    assert_eq!(output.exit_code, 2);
    assert_eq!(json["kind"], "qianji_lint_report");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["command"], "lint");
    assert_eq!(json["domain"], "bpmn");
    assert_eq!(json["ok"], false);
    assert_eq!(json["exit_code"], 2);
    assert_eq!(json["source"]["path"], json["path"]);
    assert_eq!(json["source"]["source_id"], json["report"]["source_id"]);
    assert_eq!(json["report"]["domain"], "bpmn");
    assert_eq!(json["report"]["ok"], false);
    assert_eq!(
        json["report"]["issues"][0]["code"],
        "bpmn.unsupported_gateway_configuration"
    );
    assert_eq!(
        json["report"]["issues"][0]["structured_repair"]["schema_version"],
        1
    );
    assert_eq!(
        json["report"]["issues"][0]["structured_repair"]["contract"],
        "qianji.bpmn.gateway.bounded.v1"
    );
    assert_eq!(
        json["analysis"]["repair_plans"][0]["structured_repair"]["strategy"],
        "rewrite_inclusive_gateway_to_structured_subset"
    );
}

#[test]
fn run_lint_command_keeps_gateway_condition_analysis_on_bounded_failure() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_default_condition.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_default_condition">
  <bpmn:process id="default_condition" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="verify" />
    <bpmn:exclusiveGateway id="decision" default="flow_failed" />
    <bpmn:endEvent id="failed" />
    <bpmn:endEvent id="passed" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="verify" />
    <bpmn:sequenceFlow id="flow_check" sourceRef="verify" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_failed" sourceRef="decision" targetRef="failed">
      <bpmn:conditionExpression>not verificationPassed</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_passed" sourceRef="decision" targetRef="passed">
      <bpmn:conditionExpression>verificationPassed</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::BpmnJson { path }),
        "lint JSON should render even when bounded gateway validation fails",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "lint JSON should parse",
    );

    assert_eq!(output.exit_code, 2);
    assert_eq!(json["ok"], false);
    assert_eq!(
        json["report"]["issues"][0]["structured_repair"]["strategy"],
        "make_default_branch_unconditional"
    );
    let gateway_conditions = must_some(
        json["analysis"]["gateway_conditions"].as_array(),
        "gateway condition analysis should be an array",
    );
    assert_eq!(gateway_conditions.len(), 2);
    assert_eq!(
        json["analysis"]["gateway_conditions"][0]["raw"],
        "not verificationPassed"
    );
    assert_eq!(
        json["analysis"]["gateway_conditions"][0]["parsed"]["kind"],
        "boolean_path"
    );
    assert_eq!(
        json["analysis"]["gateway_conditions"][1]["raw"],
        "verificationPassed"
    );
}

#[test]
fn run_lint_command_renders_bpmn_llm_diagnostic_with_source_span() {
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
        "lint command should render LLM output",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.unsupported_gateway_configuration")
    );
    assert!(output.rendered.contains("approved == true"));
    assert!(output.rendered.contains("rewrite this condition"));
    assert!(output.rendered.contains("Action:"));
    assert!(output.rendered.contains("Fix:"));
    assert!(output.rendered.contains("Allowed forms:"));
    assert!(output.rendered.contains("Examples:"));
    assert!(output.rendered.contains("Forbidden forms:"));
    assert!(output.rendered.contains("!approved"));
    assert!(
        output
            .rendered
            .contains("strategy: rewrite_condition_to_bounded_subset")
    );
}

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
    assert!(output.rendered.contains("qianji:outputs"));
}

#[test]
fn run_lint_command_guides_variable_to_variable_gateway_condition_repair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir
        .path()
        .join("invalid_variable_comparison_condition.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_variable_comparison">
  <bpmn:process id="section_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="present_section" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Present one section and output sectionNumber and totalSections.</qianji:prompt>
          <qianji:outputs>sectionNumber,totalSections</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="more_sections" default="flow_done" />
    <bpmn:serviceTask id="next_section" implementation="${environment.services.runAgent}" />
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
    );

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
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_questions_remaining">
  <bpmn:process id="question_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="make_question" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return JSON with currentQuestion, questionsRemaining, and sectionsRemaining.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>currentQuestion,questionsRemaining,sectionsRemaining</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
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
    );

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
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_interaction_loop">
  <bpmn:process id="interaction_loop" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_question" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return JSON with the next currentQuestion, currentChoices, and hasMoreQuestions.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>currentQuestion,currentChoices,hasMoreQuestions</qianji:outputs>
          <qianji:outputSchema name="currentChoices" kind="choice_array" value="required" label="optional" description="optional"/>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="more_questions" default="flow_done" />
    <bpmn:userTask id="ask_user">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Ask the current question.</qianji:prompt>
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
    <bpmn:sequenceFlow id="flow_decision" sourceRef="prepare_question" targetRef="more_questions" />
    <bpmn:sequenceFlow id="flow_ask" sourceRef="more_questions" targetRef="ask_user">
      <bpmn:conditionExpression>hasMoreQuestions</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="more_questions" targetRef="done" />
    <bpmn:sequenceFlow id="flow_repeat" sourceRef="ask_user" targetRef="prepare_question" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

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
    assert!(output.rendered.contains(
        "    |Help: The loop must feed answer into prepare_question before the next prompt and keep flow_done as the unconditional default exit."
    ));
    assert!(output.rendered.contains("    |Contract: qianji.bpmn.loop.progress.v1 requires in-cycle tasks to consume user feedback and emit the gateway route state."));
    assert!(output.rendered.contains("Proposed patch:"));
    assert!(output.rendered.contains("@@ -12,1 +12,1 @@"));
    assert!(
        output
            .rendered
            .contains("-          <qianji:inputs></qianji:inputs>")
    );
    assert!(
        output
            .rendered
            .contains("+          <qianji:inputs>answer</qianji:inputs>")
    );
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
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_duplicate_branch">
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

#[test]
fn run_lint_command_snapshots_redundant_user_answer_store_fix() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("redundant_answer_store.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_redundant_answer_store">
  <bpmn:process id="redundant_answer_store" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="collect_contact">
      <bpmn:extensionElements>
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
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:serviceTask id="store_contact" name="Store Contact" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Process the contact result and store it as structured contactInfo.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>contactResult</qianji:inputs>
          <qianji:outputs>contactInfo</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:userTask id="collect_visit">
      <bpmn:extensionElements>
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
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="collect_contact" />
    <bpmn:sequenceFlow id="flow_store" sourceRef="collect_contact" targetRef="store_contact" />
    <bpmn:sequenceFlow id="flow_next" sourceRef="store_contact" targetRef="collect_visit" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="collect_visit" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render redundant user-answer store fix",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.redundant_user_answer_store_service_task")
    );
    assert!(output.rendered.contains("Expected XML:"));
    assert!(
        output
            .rendered
            .contains("replace `contactInfo` with `contactResult`")
    );
    assert!(
        output
            .rendered
            .contains("store serviceTask should not invoke an LLM")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_redundant_user_answer_store_fix",
        stable_temp_output(&output.rendered, &temp_dir)
    );
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
          <qianji:outputs>sectionApproved,revisionFeedback</qianji:outputs>
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
          <qianji:outputs>userReviewApproved,specChanges</qianji:outputs>
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
          <qianji:outputs>moreQuestionsNeeded,userAnswer</qianji:outputs>
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

#[test]
fn run_lint_command_points_to_unescaped_xml_placeholder() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_prompt_placeholder.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_placeholder">
  <bpmn:process id="placeholder_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="ask">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Write a design doc to docs/YYYY-MM-DD-<topic>-design.md</qianji:prompt>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render XML placeholder diagnostic",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("bpmn.invalid_xml"));
    assert!(output.rendered.contains("<topic>"));
    assert!(output.rendered.contains("&lt;topic&gt;"));
    assert!(
        output
            .rendered
            .contains("escape raw XML-like placeholder `<topic>`")
    );
}

#[test]
fn run_lint_command_snapshots_unescaped_xml_placeholder_compact_diagnostic() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_prompt_placeholder.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_placeholder">
  <bpmn:process id="placeholder_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="ask">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Write a design doc to docs/YYYY-MM-DD-<topic>-design.md</qianji:prompt>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render XML placeholder compact diagnostic",
    );

    assert_llm_repair_snapshot_shape(
        &output.rendered,
        &[
            "escape raw XML-like placeholder `<topic>` in text",
            "target: <topic>",
            "&lt;topic&gt;",
            "escape_unescaped_xml_text_placeholder",
        ],
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_unescaped_xml_placeholder_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_snapshots_unescaped_ampersand_attribute_patch() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_choice_ampersand.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_ampersand">
  <bpmn:process id="ampersand_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <!-- Review & Confirm is allowed inside comments. -->
    <bpmn:userTask id="review">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:outputs>answer</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question>Review the details.</qianji:question>
            <qianji:choice value="confirm" label="Confirm & Submit">All information is correct</qianji:choice>
            <qianji:result output="answer"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="review" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="review" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render raw ampersand patch",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("bpmn.invalid_xml"));
    assert!(output.rendered.contains("escape raw ampersand as `&amp;`"));
    assert!(output.rendered.contains("Proposed patch:"));
    assert!(output.rendered.contains("@@ -14,1 +14,1 @@"));
    assert!(output.rendered.contains("Confirm &amp; Submit"));

    insta::assert_snapshot!(
        "qianji_lint_compact_unescaped_ampersand_attribute_patch",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_snapshots_malformed_qianji_closing_tag_patch() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("malformed_qianji_closing_tag.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_malformed_tag">
  <bpmn:process id="malformed_tag_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="screen">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:outputs>safetyScreenResult</qianji:outputs>
          <qianji:interaction type="choice">
            <qianji:question>Are any of these happening now?</qian:question>
            <qianji:choice value="routine" label="Routine">Routine appointment</qianji:choice>
            <qianji:result output="safetyScreenResult"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="screen" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="screen" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render malformed qianji closing tag patch",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("bpmn.invalid_xml"));
    assert!(
        output
            .rendered
            .contains("repair malformed XML near this token")
    );
    assert!(output.rendered.contains("Proposed patch:"));
    assert!(output.rendered.contains("</qianji:question>"));
    assert!(output.rendered.contains("@@ -12,1 +12,1 @@"));

    insta::assert_snapshot!(
        "qianji_lint_compact_malformed_qianji_closing_tag_patch",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_renders_missing_branch_condition_examples() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("missing_condition.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_missing_condition">
  <bpmn:process id="condition_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:exclusiveGateway id="decision" default="flow_default" />
    <bpmn:endEvent id="approved_end" />
    <bpmn:endEvent id="fallback_end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_yes" sourceRef="decision" targetRef="approved_end" />
    <bpmn:sequenceFlow id="flow_default" sourceRef="decision" targetRef="fallback_end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render missing condition examples",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("add_missing_branch_condition"));
    assert!(
        output
            .rendered
            .contains("promote_unconditional_branch_to_default")
    );
    assert!(output.rendered.contains("Allowed forms:"));
    assert!(output.rendered.contains("Examples:"));
    assert!(output.rendered.contains("not approved"));
    assert!(output.rendered.contains("risk >= 7"));
}

#[test]
fn run_lint_command_snapshots_missing_branch_condition_compact_diagnostic() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("missing_condition.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_missing_condition">
  <bpmn:process id="condition_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:exclusiveGateway id="decision" default="flow_default" />
    <bpmn:endEvent id="approved_end" />
    <bpmn:endEvent id="fallback_end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_yes" sourceRef="decision" targetRef="approved_end" />
    <bpmn:sequenceFlow id="flow_default" sourceRef="decision" targetRef="fallback_end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render missing condition compact diagnostic",
    );

    assert_llm_repair_snapshot_shape(
        &output.rendered,
        &[
            "add a bounded conditionExpression inside this non-default branch",
            "Allowed forms:",
            "conditionExpression on the default branch",
            "add_missing_branch_condition",
        ],
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_missing_branch_condition_repair",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_json_reports_gateway_condition_structure() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("valid_condition.bpmn");
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
      <bpmn:conditionExpression>approved</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_default" sourceRef="decision" targetRef="fallback_end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::BpmnJson { path }),
        "lint command should render JSON output",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "lint JSON should parse",
    );

    assert_eq!(output.exit_code, 0);
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["command"], "lint");
    assert_eq!(json["domain"], "bpmn");
    assert_eq!(json["ok"], true);
    assert_eq!(json["exit_code"], 0);
    assert_eq!(
        json["analysis"]["gateway_conditions"][0]["source_ref"],
        "decision"
    );
    assert_eq!(
        json["analysis"]["gateway_conditions"][0]["parsed"]["kind"],
        "boolean_path"
    );
    assert_eq!(
        json["analysis"]["gateway_conditions"][0]["parsed"]["path"],
        "approved"
    );
}

#[test]
fn run_lint_command_renders_bpmn_snapshot_evidence() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_lane.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_invalid_lane_set">
  <bpmn:process id="lane_flow" isExecutable="true">
    <bpmn:laneSet id="lane_set_ops">
      <bpmn:lane id="lane_ops" name="Operations">
        <bpmn:flowNodeRef>review</bpmn:flowNodeRef>
      </bpmn:lane>
    </bpmn:laneSet>
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="review" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render snapshot evidence",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("bpmn.unsupported_lane_surface"));
    assert!(output.rendered.contains("\"snapshot_available\": true"));
    assert!(output.rendered.contains("\"lane_set_count\": 1"));
    assert!(output.rendered.contains("\"flow_node_refs\""));
    assert!(output.rendered.contains("\"review\""));
}

#[test]
fn run_lint_command_renders_success_for_valid_dmn() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("valid.dmn");
    write_file(
        &path,
        r#"<definitions xmlns="https://www.omg.org/spec/DMN/20191111/MODEL/"
  id="Definitions_loan"
  name="Loan DRD"
  namespace="http://example.com/dmn">
  <decision id="loan-decision" name="Loan Decision">
    <decisionTable id="decision_table_1" hitPolicy="UNIQUE">
      <input id="input_1" label="tier">
        <inputExpression id="input_expression_1" typeRef="string">
          <text>tier</text>
        </inputExpression>
      </input>
      <output id="output_1" name="approval" label="approval" typeRef="string" />
      <rule id="rule_approve">
        <inputEntry id="input_entry_1">
          <text>"gold"</text>
        </inputEntry>
        <outputEntry id="output_entry_1">
          <text>"approve"</text>
        </outputEntry>
      </rule>
      <rule id="rule_review">
        <inputEntry id="input_entry_2">
          <text>-</text>
        </inputEntry>
        <outputEntry id="output_entry_2">
          <text>"review"</text>
        </outputEntry>
      </rule>
    </decisionTable>
  </decision>
</definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Dmn { path }),
        "lint command should render success output",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("[ok]"));
    assert!(output.rendered.contains("dmn"));
    assert!(output.rendered.contains("no blocking issues"));
}
