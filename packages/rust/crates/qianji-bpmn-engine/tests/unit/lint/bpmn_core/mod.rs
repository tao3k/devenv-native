use super::{
    BpmnSourceFile, LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source,
};

mod boundary;
mod document_surface;
mod gateway;
mod subprocess;

#[test]
fn bpmn_linter_uses_parser_offset_for_unescaped_placeholder_diagnostic() {
    let contents = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  id="pkg_placeholder">
  <bpmn:process id="placeholder_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="ask">
      <bpmn:documentation>Write a design doc to docs/YYYY-MM-DD-<topic>-design.md</bpmn:documentation>
    </bpmn:serviceTask>
  </bpmn:process>
</bpmn:definitions>"#;
    let placeholder_start = contents
        .find("<topic>")
        .unwrap_or_else(|| panic!("fixture should include raw placeholder token"));
    let parser_error_offset = contents
        .find("</bpmn:documentation>")
        .unwrap_or_else(|| panic!("fixture should include parser-visible mismatched closing tag"));
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid_prompt_placeholder.bpmn",
        contents,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_xml");
    assert_eq!(
        issue.evidence["parser_offset"].as_u64(),
        Some(
            u64::try_from(parser_error_offset)
                .unwrap_or_else(|error| panic!("parser offset should fit u64: {error}"))
        )
    );
    let Some(source_diagnostic) = issue.source_diagnostic.as_ref() else {
        panic!("invalid XML placeholder should carry source diagnostic");
    };
    assert_eq!(source_diagnostic.span.start, placeholder_start);
    assert_eq!(
        source_diagnostic.span.end,
        placeholder_start + "<topic>".len()
    );
    assert!(source_diagnostic.label.contains("<topic>"));
    let Some(structured_repair) = issue.structured_repair.as_ref() else {
        panic!("invalid XML placeholder should carry structured repair");
    };
    assert_eq!(
        structured_repair["strategy"],
        "escape_unescaped_xml_text_placeholder"
    );
}

#[test]
fn bpmn_linter_points_to_unescaped_ampersand_attribute() {
    let contents = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  targetNamespace="https://example.test/bpmn">
  <bpmn:process id="ampersand_flow" isExecutable="true">
    <bpmn:startEvent id="start"/>
    <!-- Review & Confirm is allowed inside comments. -->
    <bpmn:userTask id="review">
      <bpmn:ioSpecification>
        <bpmn:dataInput id="review_choice" name="Confirm & Submit"/>
      </bpmn:ioSpecification>
    </bpmn:userTask>
    <bpmn:endEvent id="done"/>
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="review"/>
    <bpmn:sequenceFlow id="flow_done" sourceRef="review" targetRef="done"/>
  </bpmn:process>
</bpmn:definitions>"#;
    let raw_ampersand = contents.find("Confirm & Submit").map_or_else(
        || panic!("fixture should include raw ampersand attribute"),
        |offset| offset + "Confirm ".len(),
    );
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid_choice_ampersand.bpmn",
        contents,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_xml");
    if let Some(source_diagnostic) = issue.source_diagnostic.as_ref() {
        assert_eq!(source_diagnostic.span.start, raw_ampersand);
        assert_eq!(source_diagnostic.span.end, raw_ampersand + 1);
        assert!(source_diagnostic.label.contains("&amp;"));
    }
    assert!(
        issue.llm_fix_prompt.contains("&amp;") || issue.title.contains("well-formed"),
        "{issue:#?}"
    );
}

#[test]
fn bpmn_linter_does_not_escape_real_xml_elements_for_malformed_tags() {
    let contents = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  targetNamespace="https://example.test/bpmn">
  <bpmn:process id="malformed_flow" isExecutable="true">
    <bpmn:userTask id="review">
      <bpmn:documentation>Review the choice.</bpmn:documntation>
    </bpmn:userTask>
  </bpmn:process>
</bpmn:definitions>"#;
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "malformed_documentation.bpmn",
        contents,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_xml");
    let rendered = serde_json::to_string(issue)
        .unwrap_or_else(|error| panic!("issue should serialize: {error}"));
    assert!(!rendered.contains("escape raw XML-like placeholder `<extensionElements>`"));
    assert!(!rendered.contains("escape_text_node_placeholder"));
    assert!(
        rendered.contains("repair_malformed_xml_closing_tag")
            || rendered.contains("repair_malformed_xml_token")
    );
    assert!(
        issue
            .structured_repair
            .as_ref()
            .and_then(|repair| repair.get("line_fixes"))
            .and_then(serde_json::Value::as_array)
            .and_then(|line_fixes| line_fixes.first())
            .and_then(|line_fix| line_fix.get("xml"))
            .and_then(serde_json::Value::as_str)
            .is_none_or(|xml| xml.contains("</bpmn:documentation>"))
    );
}
