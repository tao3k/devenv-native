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
</bpmn:definitions>"#;
    let placeholder_start = contents
        .find("<topic>")
        .unwrap_or_else(|| panic!("fixture should include raw placeholder token"));
    let parser_error_offset = contents
        .find("</qianji:prompt>")
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
fn bpmn_linter_does_not_escape_real_xml_elements_for_malformed_tags() {
    let contents = r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
             xmlns:qianji="https://qianji.dev/bpmn/extensions">
  <process id="malformed_flow" isExecutable="true">
    <userTask id="review">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Review the choice.</qianji:prompt>
          <qianji:interaction type="choice">
            <qianji:choice value="implement">Implement anyway.</q:ji:choice>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
  </process>
</definitions>"#;
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "malformed_qianji_choice.bpmn",
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
    assert!(rendered.contains("repair_malformed_xml_token"));
}
