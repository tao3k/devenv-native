use super::*;

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
            .contains("requires exactly one of `--bpmn <path>` or `--dmn <path>`")
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
    assert!(output.rendered.contains("[bpmn.unsupported_element]"));
    assert!(output.rendered.contains("### Repair Guidance"));
    assert!(output.rendered.contains("### LLM Fix Prompt"));
    assert!(output.rendered.contains("inclusiveGateway"));
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
