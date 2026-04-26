use super::*;

const VALID_WORKFLOW_PLAN: &str = r#"{
  "version": 1,
  "name": "approval-plan",
  "constructs": [
    "service-task.agent",
    "user-task.interaction",
    "gateway.exclusive.bounded"
  ],
  "tasks": [
    {
      "id": "Task_Check",
      "construct": "service-task.agent",
      "outputs": ["ready"]
    },
    {
      "id": "Task_Approve",
      "construct": "user-task.interaction",
      "inputs": ["ready"],
      "outputs": ["approved"]
    }
  ],
  "edges": [
    {"from": "start", "to": "Task_Check"},
    {"from": "Task_Check", "to": "Task_Approve"},
    {"from": "Task_Approve", "to": "end", "condition": "approved"}
  ]
}"#;

const INVALID_WORKFLOW_PLAN: &str = r#"{
  "version": 1,
  "name": "broken-plan",
  "constructs": ["service-task.agent"],
  "tasks": [
    {
      "id": "Task_Check",
      "construct": "service-task.agent",
      "outputs": ["ready"]
    }
  ],
  "edges": [
    {"from": "Task_Check", "to": "end", "condition": "${approved == true}"}
  ]
}"#;

#[test]
fn parse_lint_command_accepts_inferred_target() {
    let command = must_some(
        must_ok(
            parse_lint_command(&to_args(&["qianji", "lint", "plan.json"])),
            "lint parse should succeed",
        ),
        "lint command should be detected",
    );

    assert_eq!(
        command,
        LintCliCommand::Auto {
            path: PathBuf::from("plan.json")
        }
    );
}

#[test]
fn parse_lint_command_accepts_bpmn_target() {
    let command = must_some(
        must_ok(
            parse_lint_command(&to_args(&[
                "qianji",
                "lint",
                "--bpmn",
                "fixtures/sample.bpmn",
            ])),
            "lint parse should succeed",
        ),
        "lint command should be detected",
    );

    assert_eq!(
        command,
        LintCliCommand::Bpmn {
            path: PathBuf::from("fixtures/sample.bpmn")
        }
    );
}

#[test]
fn parse_lint_command_accepts_linter_alias_for_dmn_target() {
    let command = must_some(
        must_ok(
            parse_lint_command(&to_args(&[
                "qianji",
                "linter",
                "--dmn",
                "fixtures/sample.dmn",
            ])),
            "linter alias parse should succeed",
        ),
        "linter alias should be detected",
    );

    assert_eq!(
        command,
        LintCliCommand::Dmn {
            path: PathBuf::from("fixtures/sample.dmn")
        }
    );
}

#[test]
fn parse_lint_command_rejects_mixed_targets() {
    let error = match parse_lint_command(&to_args(&[
        "qianji", "lint", "--bpmn", "a.bpmn", "--dmn", "b.dmn",
    ])) {
        Ok(command) => panic!("mixed lint targets should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("requires exactly one target path")
    );
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
    assert!(output.rendered.starts_with("# Lint Failed"));
    assert!(
        output
            .rendered
            .contains("[bpmn.unsupported_gateway_configuration]")
    );
    assert!(output.rendered.contains("### Repair Guidance"));
    assert!(output.rendered.contains("### LLM Fix Prompt"));
    assert!(output.rendered.contains("inclusiveGateway"));
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
    assert!(output.rendered.contains("[bpmn.unsupported_lane_surface]"));
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
    assert!(output.rendered.starts_with("# Lint Passed"));
    assert!(output.rendered.contains("Domain: dmn"));
    assert!(output.rendered.contains("no blocking issues"));
}

#[test]
fn run_lint_command_infers_valid_workflow_plan() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("plan.json");
    write_file(&path, VALID_WORKFLOW_PLAN);

    let output = must_ok(
        run_lint_command(LintCliCommand::Auto { path }),
        "lint command should render WorkflowPlan success output",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# Lint Passed"));
    assert!(output.rendered.contains("Domain: workflow-plan"));
}

#[test]
fn run_lint_command_infers_invalid_workflow_plan() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("plan.json");
    write_file(&path, INVALID_WORKFLOW_PLAN);

    let output = must_ok(
        run_lint_command(LintCliCommand::Auto { path }),
        "lint command should render WorkflowPlan failure output",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.starts_with("# Lint Failed"));
    assert!(output.rendered.contains("Domain: workflow-plan"));
    assert!(
        output
            .rendered
            .contains("construct_plan.gateway_construct_not_selected")
    );
    assert!(
        output
            .rendered
            .contains("construct_plan.unsupported_condition")
    );
}

#[test]
fn run_lint_command_rejects_duplicate_workflow_plan_constructs() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("plan.json");
    write_file(
        &path,
        r#"{
  "version": 1,
  "name": "duplicate-constructs",
  "constructs": ["service-task.agent", "service-task.agent"],
  "tasks": [
    {
      "id": "Task_DoWork",
      "construct": "service-task.agent",
      "outputs": ["result"]
    }
  ],
  "edges": [
    {"from": "start", "to": "Task_DoWork"},
    {"from": "Task_DoWork", "to": "end"}
  ]
}"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Auto { path }),
        "duplicate constructs should render WorkflowPlan lint output",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.starts_with("# Lint Failed"));
    assert!(
        output
            .rendered
            .contains("construct_plan.duplicate_construct")
    );
    assert!(output.rendered.contains("Treat `constructs` as a set"));
}

#[test]
fn run_lint_command_reports_workflow_plan_parse_errors_as_lint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("plan.json");
    write_file(
        &path,
        r#"{
  "version": "1",
  "plan": {
    "nodes": []
  }
}"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Auto { path }),
        "workflow-plan parse errors should render lint output",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.starts_with("# Lint Failed"));
    assert!(output.rendered.contains("Domain: workflow-plan"));
    assert!(
        output
            .rendered
            .contains("construct_plan.invalid_json_shape")
    );
    assert!(output.rendered.contains("\"version\": 1"));
    assert!(output.rendered.contains("do not use `nodes`"));
    assert!(output.rendered.contains("Treat `constructs` as a set"));
}
