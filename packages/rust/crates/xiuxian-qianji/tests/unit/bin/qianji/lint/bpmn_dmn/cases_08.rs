use super::*;

#[test]
fn run_lint_command_points_to_unescaped_xml_placeholder() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_prompt_placeholder.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_placeholder">
  <bpmn:process id="placeholder_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="ask">
      <bpmn:documentation>Write a design doc to docs/YYYY-MM-DD-<topic>-design.md</bpmn:documentation>
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
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_placeholder">
  <bpmn:process id="placeholder_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="ask">
      <bpmn:documentation>Write a design doc to docs/YYYY-MM-DD-<topic>-design.md</bpmn:documentation>
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
fn run_lint_command_snapshots_unescaped_ampersand_in_native_assignment_patch() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("invalid_choice_ampersand.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_ampersand">
  <bpmn:process id="ampersand_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <!-- Review & Confirm is allowed inside comments. -->
    <bpmn:userTask id="review">
      <bpmn:documentation>Review the details.</bpmn:documentation>
      <bpmn:ioSpecification>
        <bpmn:dataInput id="review_input_interactionType" name="interactionType" />
        <bpmn:dataInput id="review_input_choices" name="choices" />
        <bpmn:dataOutput id="review_output_answer" name="answer" />
        <bpmn:inputSet id="review_input_set">
          <bpmn:dataInputRefs>review_input_interactionType</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>review_input_choices</bpmn:dataInputRefs>
        </bpmn:inputSet>
        <bpmn:outputSet id="review_output_set">
          <bpmn:dataOutputRefs>review_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataInputAssociation>
        <bpmn:assignment>
          <bpmn:from>choice</bpmn:from>
          <bpmn:to>review_input_interactionType</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:assignment>
          <bpmn:from>[{"value":"confirm","label":"Confirm & Submit"}]</bpmn:from>
          <bpmn:to>review_input_choices</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>review_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>answer</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
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
    assert!(
        output
            .rendered
            .contains("repair malformed XML near this token")
    );
    assert!(output.rendered.contains("repair_xml_tag_or_nesting"));
    assert!(output.rendered.contains("Confirm & Submit"));

    insta::assert_snapshot!(
        "qianji_lint_compact_unescaped_ampersand_assignment_patch",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}

#[test]
fn run_lint_command_snapshots_malformed_native_closing_tag_patch() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let path = temp_dir.path().join("malformed_native_closing_tag.bpmn");
    write_file(
        &path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_malformed_tag">
  <bpmn:process id="malformed_tag_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="screen">
      <bpmn:documentation>Are any of these happening now?</bpmn:doc>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="screen" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="screen" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );

    let output = must_ok(
        run_lint_command(LintCliCommand::Bpmn { path }),
        "lint command should render malformed native closing tag patch",
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.rendered.contains("bpmn.invalid_xml"));
    assert!(
        output
            .rendered
            .contains("repair malformed XML near this token")
    );
    assert!(output.rendered.contains("repair_xml_tag_or_nesting"));
    assert!(output.rendered.contains("</bpmn:doc>"));

    insta::assert_snapshot!(
        "qianji_lint_compact_malformed_native_closing_tag_patch",
        stable_temp_output(&output.rendered, &temp_dir)
    );
}
