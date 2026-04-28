use super::*;

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
