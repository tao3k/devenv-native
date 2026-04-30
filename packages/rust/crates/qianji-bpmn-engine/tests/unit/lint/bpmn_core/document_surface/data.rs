use super::*;

#[test]
fn bpmn_linter_accepts_process_data_object_execution_surface() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-data-object-reference.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_reports_data_state_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-data-state.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_data_surface");
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(issue.evidence["snapshot"]["data_store_count"], 1);
    assert_eq!(
        issue.evidence["snapshot"]["data_stores"][0]["data_state"]["name"],
        "archived"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["data_objects"][0]["data_state"]["data_state_id"],
        "DataState_ObjectDraft"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["data_object_references"][0]["data_state"]["name"],
        "submitted"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["data_store_references"][0]["data_state"]["name"],
        "available"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["io_specifications"][0]["data_inputs"][0]["data_state"]
            ["name"],
        "received"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["io_specifications"][0]["data_outputs"][0]["data_state"]
            ["name"],
        "approved"
    );
}

#[test]
fn bpmn_linter_reports_task_data_association_transformation_with_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "metadata-data-association-expressions.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_task_configuration");
    assert_eq!(issue.evidence["detail"], "task_io_transformation_deferred");
    assert!(issue.summary.contains("Task"));
}

#[test]
fn bpmn_linter_reports_io_set_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-io-sets.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_data_surface");
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["io_specifications"][0]["input_sets"][0]["optional_input_refs"]
            [0],
        "ProcessInput_Optional"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["io_specifications"][0]["input_sets"][0]["while_executing_input_refs"]
            [0],
        "ProcessInput_Stream"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["io_specifications"][0]["output_sets"][0]["optional_output_refs"]
            [0],
        "ProcessOutput_Optional"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_data"][0]["io_specifications"][0]["output_sets"][0]["while_executing_output_refs"]
            [0],
        "ProcessOutput_Stream"
    );
}
