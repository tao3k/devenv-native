use super::*;

#[test]
fn bpmn_linter_reports_flow_element_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-flow-element.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_flow_element_metadata");
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert!(
        issue
            .why_it_failed
            .contains("category classification, scheduling")
    );
    let Some(structured_repair) = issue.structured_repair.as_ref() else {
        panic!("flow-element metadata issue should include structured repair");
    };
    assert_eq!(
        structured_repair["contract"],
        "bpmn.native.flow_element.metadata_only.v1"
    );
    assert_eq!(
        issue.evidence["snapshot"]["flow_element_metadata"]["element_count"],
        3
    );
    assert_eq!(
        issue.evidence["snapshot"]["flow_element_metadata"]["auditing_count"],
        2
    );
    assert_eq!(
        issue.evidence["snapshot"]["flow_element_metadata"]["monitoring_count"],
        2
    );
    assert_eq!(
        issue.evidence["snapshot"]["flow_element_metadata"]["category_value_ref_count"],
        3
    );
    assert_eq!(
        issue.evidence["snapshot"]["flow_element_metadata"]["processes"][0]["flow_elements"][1]["element_id"],
        "Task_Review"
    );
    assert_eq!(
        issue.evidence["snapshot"]["flow_element_metadata"]["processes"][0]["flow_elements"][1]["category_value_refs"]
            [1],
        "CategoryValue_Monitoring"
    );
}
