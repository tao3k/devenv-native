use super::*;

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
    assert!(
        output
            .rendered
            .contains("add_condition_expression_to_non_default_branch")
    );
    assert!(
        output
            .rendered
            .contains("promote_unconditional_branch_to_default")
    );
    assert!(output.rendered.contains("Allowed forms:"));
    assert!(output.rendered.contains("Examples:"));
    assert!(output.rendered.contains("not approved"));
    assert!(output.rendered.contains("amount > 100"));
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
            "Add a child `conditionExpression`",
            "Allowed forms:",
            "conditionExpression on the default branch",
            "add_condition_expression_to_non_default_branch",
            "resolve_unconditional_non_default_branch",
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
    let path = temp_dir.path().join("invalid_data_surface.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_invalid_data_surface">
  <bpmn:dataObject id="order_payload" name="Order Payload" />
  <bpmn:process id="data_surface_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="review" />
    <bpmn:dataObjectReference id="order_payload_ref" dataObjectRef="order_payload" />
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
    assert!(output.rendered.contains("bpmn.unsupported_data_surface"));
    assert!(output.rendered.contains("\"snapshot_available\": true"));
    assert!(
        output
            .rendered
            .contains("\"data_object_reference_count\": 1")
    );
    assert!(
        output
            .rendered
            .contains("\"data_object_ref\": \"order_payload\"")
    );
    assert!(output.rendered.contains("\"order_payload\""));
}
