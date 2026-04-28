use super::*;
use qianji_bpmn_engine::{BpmnSourceFile, lint_bpmn_source};

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
fn parse_emit_command_accepts_bpmn_target() {
    let command = must_some(
        must_ok(
            parse_emit_command(&to_args(&["qianji", "emit", "plan.json", "--bpmn"])),
            "emit parse should succeed",
        ),
        "emit command should be detected",
    );

    assert_eq!(
        command,
        EmitCliCommand::Bpmn {
            path: PathBuf::from("plan.json")
        }
    );
}

#[test]
fn run_emit_command_renders_lint_clean_bpmn() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should be created");
    let path = temp_dir.path().join("valid-plan.json");
    write_file(&path, VALID_WORKFLOW_PLAN);

    let output = must_ok(
        run_emit_command(&EmitCliCommand::Bpmn { path }),
        "emit bpmn should render",
    );
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "emit-plan.bpmn".to_string(),
        output.rendered.clone(),
    ));

    assert!(output.rendered.contains("<definitions"));
    assert!(
        output
            .rendered
            .contains("xmlns:qianji=\"https://qianji.dev/bpmn/extensions\"")
    );
    assert!(
        output
            .rendered
            .contains("<exclusiveGateway id=\"Gateway_Task_Approve\"")
    );
    assert!(report.ok, "emitted BPMN should lint clean: {report:?}");
}

#[test]
fn run_emit_command_rejects_invalid_plan() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should be created");
    let path = temp_dir.path().join("invalid-plan.json");
    write_file(&path, INVALID_WORKFLOW_PLAN);

    let result = run_emit_command(&EmitCliCommand::Bpmn { path });
    let Err(error) = result else {
        panic!("invalid construct plan should not emit BPMN");
    };
    let message = error.to_string();

    assert!(message.contains("Status: failed"));
    assert!(message.contains("construct_plan.gateway_construct_not_selected"));
    assert!(message.contains("construct_plan.unsupported_condition"));
}
