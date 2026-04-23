use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_dmndi_only_document_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("metadata-only-dmndi-20191111.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_dmndi_document");
    assert!(issue.title.contains("diagram interchange"));
    assert!(issue.summary.contains("<dmndi:DMNDI>"));
    assert!(
        issue
            .why_it_failed
            .contains("DMNDI blocks as diagram-interchange metadata only")
    );
    assert!(issue.why_it_failed.contains("DMNDI metadata"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent rules"))
    );
    assert!(issue.llm_fix_prompt.contains(
        "do not fabricate decision-table logic just from top-level `<dmndi:DMNDI>` metadata"
    ));
    assert_eq!(issue.evidence["dmndi_count"], json!(1));
    assert_eq!(issue.evidence["document_root"]["dmndi_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["dmndi_id"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"],
        json!([])
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}

#[test]
fn dmn_linter_surfaces_direct_dmndi_diagram_element_metadata() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-dmndi-diagram-elements-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_dmndi_document");
    assert!(issue.summary.contains("DMNShape"));
    assert!(issue.summary.contains("DMNEdge"));
    assert!(
        issue
            .summary
            .contains("dc:Bounds x '120', y '80', width '180', height '80'")
    );
    assert!(
        issue
            .summary
            .contains("2 waypoint(s) [di:waypoint x '180', y '120'; di:waypoint x '300', y '120']")
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["diagram_id"],
        json!("diagram_metadata")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["shape_id"],
        json!("shape_input_1")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["dmn_element_ref"],
        json!("InputData_1")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["is_listed_input_data"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["bounds"]["x"],
        json!("120")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["bounds"]["y"],
        json!("80")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["bounds"]["width"],
        json!("180")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["bounds"]["height"],
        json!("80")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["edge_id"],
        json!("edge_requirement_1")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["dmn_element_ref"],
        json!("Requirement_1")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["waypoints"]
            [0]["x"],
        json!("180")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["waypoints"]
            [0]["y"],
        json!("120")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["waypoints"]
            [1]["x"],
        json!("300")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["waypoints"]
            [1]["y"],
        json!("120")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["label"],
        json!(null)
    );
}

#[test]
fn dmn_linter_surfaces_listed_input_data_shape_metadata() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-dmndi-listed-input-shape-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_dmndi_document");
    assert!(issue.summary.contains("isListedInputData true"));
    assert!(
        issue
            .summary
            .contains("dc:Bounds x '24', y '18', width '160', height '40'")
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["shape_id"],
        json!("shape_input_listed_1")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["dmn_element_ref"],
        json!("InputData_1")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["is_listed_input_data"],
        json!(true)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["bounds"]["x"],
        json!("24")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["bounds"]["height"],
        json!("40")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"],
        json!(null)
    );
}

#[test]
fn dmn_linter_surfaces_direct_label_placeholders() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-dmndi-label-placeholders-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_dmndi_document");
    assert!(issue.summary.contains("DMNLabel id 'shape_label_1'"));
    assert!(issue.summary.contains("DMNLabel id 'edge_label_1'"));
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["bounds"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"]["label_id"],
        json!("shape_label_1")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"]["bounds"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"]["text"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["label"]["label_id"],
        json!("edge_label_1")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["label"]["bounds"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["label"]["text"],
        json!(null)
    );
}

#[test]
fn dmn_linter_surfaces_direct_label_bounds_placeholders() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-dmndi-label-bounds-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_dmndi_document");
    assert!(issue.summary.contains(
        "DMNLabel id 'shape_label_1', dc:Bounds x '33', y '14', width '49', height '10'"
    ));
    assert!(issue.summary.contains(
        "DMNLabel id 'edge_label_1', dc:Bounds x '300', y '120', width '42', height '12'"
    ));
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"]["bounds"]
            ["x"],
        json!("33")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"]["bounds"]
            ["height"],
        json!("10")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["label"]["bounds"]
            ["x"],
        json!("300")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["label"]["bounds"]
            ["width"],
        json!("42")
    );
}

#[test]
fn dmn_linter_surfaces_direct_label_text_payloads() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-dmndi-label-text-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_dmndi_document");
    assert!(issue.summary.contains("text 'Shape Label'"));
    assert!(issue.summary.contains("text 'Edge Label'"));
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["bounds"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"]["bounds"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["shapes"][0]["label"]["text"],
        json!("Shape Label")
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["label"]["bounds"],
        json!(null)
    );
    assert_eq!(
        issue.evidence["document_root"]["dmndi_blocks"][0]["diagrams"][0]["edges"][0]["label"]["text"],
        json!("Edge Label")
    );
}
