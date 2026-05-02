use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

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
fn bpmn_linter_reports_data_store_reference_bindings_with_evidence() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-data-store-binding.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_data_store_binding");
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(issue.evidence["snapshot"]["data_store_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["data_store_binding_count"], 2);

    let bindings = issue.evidence["snapshot"]["data_store_bindings"]
        .as_array()
        .unwrap_or_else(|| panic!("data-store binding evidence should be an array"));
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0]["process_id"], "Process_DataStoreBinding");
    assert_eq!(bindings[0]["association_kind"], "dataInputAssociation");
    assert_eq!(bindings[0]["association_id"], "Association_ReadStore");
    assert_eq!(bindings[0]["usage"], "sourceRef");
    assert_eq!(
        bindings[0]["data_store_reference_id"],
        "DataStoreReference_Orders"
    );
    assert_eq!(bindings[0]["data_store_ref"], "DataStore_Orders");
    assert_eq!(bindings[1]["association_kind"], "dataOutputAssociation");
    assert_eq!(bindings[1]["association_id"], "Association_WriteStore");
    assert_eq!(bindings[1]["usage"], "targetRef");
    assert_eq!(
        bindings[1]["data_store_reference_id"],
        "DataStoreReference_Orders"
    );
    assert!(
        issue.why_it_failed.contains("persistent read or write"),
        "{issue:#?}"
    );
    assert!(issue.llm_fix_prompt.contains("workflow variables"));
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
