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
