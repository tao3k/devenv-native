use super::*;

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
fn run_lint_command_snapshots_missing_service_task_tool_scope() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("missing_tool_scope.bpmn");
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
        "lint command should render missing tool-scope diagnostic",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.service_task.tool_scope.missing")
    );
    assert!(
        output
            .rendered
            .contains("qianji.bpmn.service_task.tool_scope.v1")
    );
    assert!(output.rendered.contains("Expected XML:"));
    assert!(
        output
            .rendered
            .contains("tool=\"bash\" command=\"npm test\"")
    );
    assert!(!output.rendered.contains("\nAction:"));
    assert!(!output.rendered.contains("\nFix:"));

    insta::assert_snapshot!(
        "qianji_lint_compact_missing_service_task_tool_scope",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_snapshots_incomplete_service_task_tool_scope() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("incomplete_tool_scope.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_tool_scope">
  <bpmn:process id="tool_scope" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="write_report" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Write the report file and return reportPath.</qianji:prompt>
          <qianji:tools>write</qianji:tools>
          <qianji:toolScope tool="write"/>
          <qianji:inputs>summary</qianji:inputs>
          <qianji:outputs>reportPath</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="write_report" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="write_report" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render incomplete tool-scope diagnostic",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.service_task.tool_scope.incomplete")
    );
    assert!(
        output
            .rendered
            .contains("<qianji:toolScope tool=\"write\" path=\"docs/**\"/>")
    );

    insta::assert_snapshot!(
        "qianji_lint_compact_incomplete_service_task_tool_scope",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_rejects_undeclared_tool_scope() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("undeclared_tool_scope.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_tool_scope">
  <bpmn:process id="tool_scope" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="summarize" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Summarize the declared input and return summary.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:toolScope tool="read" path="docs/**"/>
          <qianji:inputs>notes</qianji:inputs>
          <qianji:outputs>summary</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="summarize" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="summarize" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render undeclared tool-scope diagnostic",
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .rendered
            .contains("bpmn.service_task.tool_scope.undeclared")
    );
    assert!(output.rendered.contains("scope tool is not declared"));
}
