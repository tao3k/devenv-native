use super::*;

#[test]
fn bpmn_linter_reports_data_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-data-object-reference.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_data_surface");
    assert!(issue.why_it_failed.contains("JSON variables"));
    assert!(issue.llm_fix_prompt.contains("DMN inputs"));
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(issue.evidence["snapshot"]["item_definition_count"], 1);
    assert_eq!(
        issue.evidence["snapshot"]["item_definitions"][0]["item_definition_id"],
        "order_item"
    );
    assert_eq!(issue.evidence["snapshot"]["data_object_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["data_object_reference_count"], 1);
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["data_object_references"][0]["data_object_ref"],
        "order_payload"
    );
}
