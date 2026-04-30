use super::*;
use qianji_bpmn_engine::{
    BpmnParseOptions, BpmnSourceFile, lint_bpmn_source, parse_bpmn_package, snapshot_bpmn_source,
};

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
fn run_emit_command_renders_native_bpmn_with_standard_di() {
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
    let source = BpmnSourceFile::new("emit-plan.bpmn".to_string(), output.rendered.clone());
    let snapshot = must_ok(
        snapshot_bpmn_source(&source),
        "emitted BPMN should snapshot cleanly",
    );
    must_ok(
        parse_bpmn_package(&[source], &BpmnParseOptions::default()),
        "emitted BPMN should parse cleanly",
    );

    assert!(output.rendered.contains("<definitions"));
    assert!(!output.rendered.contains("xmlns:qianji"));
    assert!(output.rendered.contains("<ioSpecification>"));
    assert!(output.rendered.contains("xmlns:bpmndi"));
    assert!(output.rendered.contains("<bpmndi:BPMNDiagram"));
    assert!(output.rendered.contains("<dc:Bounds"));
    assert!(output.rendered.contains("<di:waypoint"));
    assert!(
        output
            .rendered
            .contains("<exclusiveGateway id=\"Gateway_Task_Approve\"")
    );
    assert_eq!(snapshot.root.diagram_count, 1);
    let plane = snapshot.root.diagrams[0]
        .plane
        .as_ref()
        .unwrap_or_else(|| panic!("emitted BPMN should preserve a BPMNPlane"));
    assert_eq!(plane.shapes.len(), 5);
    assert_eq!(plane.edges.len(), 4);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].code, "bpmn.metadata_di_surface");
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
