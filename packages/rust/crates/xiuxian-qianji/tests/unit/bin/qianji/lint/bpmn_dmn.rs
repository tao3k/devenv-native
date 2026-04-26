use super::*;

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
