use super::{LintCliCommand, TempDir, must_ok, run_lint_command, write_file};

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
    assert!(output.rendered.starts_with("[ok]"));
    assert!(output.rendered.contains("workflow-plan"));
}

#[test]
fn run_lint_command_renders_workflow_plan_json_envelope() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("plan.json");
    write_file(&path, VALID_WORKFLOW_PLAN);

    let output = must_ok(
        run_lint_command(LintCliCommand::AutoJson { path }),
        "workflow-plan lint command should render JSON output",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "workflow-plan lint JSON should parse",
    );

    assert_eq!(output.exit_code, 0);
    assert_eq!(json["kind"], "qianji_lint_report");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["command"], "lint");
    assert_eq!(json["domain"], "workflow_plan");
    assert_eq!(json["ok"], true);
    assert_eq!(json["exit_code"], 0);
    assert_eq!(json["source"]["path"], json["path"]);
    assert_eq!(json["source"]["source_id"], json["report"]["source_id"]);
    assert_eq!(json["report"]["domain"], "workflow_plan");
    assert_eq!(json["report"]["ok"], true);
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
    assert!(output.rendered.starts_with("[lint:error]"));
    assert!(output.rendered.contains("workflow-plan"));
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
    assert!(output.rendered.starts_with("[lint:error]"));
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
    assert!(output.rendered.starts_with("[lint:error]"));
    assert!(output.rendered.contains("workflow-plan"));
    assert!(
        output
            .rendered
            .contains("construct_plan.invalid_json_shape")
    );
    assert!(output.rendered.contains("numeric version"));
    assert!(output.rendered.contains("constructs, tasks, and edges"));
}
