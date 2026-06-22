use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};
use xiuxian_qianji_bpmn_engine::snapshot_bpmn_source;

#[test]
fn bpmn_linter_accepts_process_data_object_execution_surface() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-data-object-reference.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_data_state_metadata_surface() {
    let source = bpmn_fixture_source("metadata-data-state.bpmn");
    let report = lint_bpmn_source(&source);

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "data metadata should lint cleanly: {report:?}");
    assert!(report.issues.is_empty());

    let snapshot = snapshot_bpmn_source(&source)
        .unwrap_or_else(|error| panic!("data-state fixture should snapshot: {error}"));
    assert_eq!(snapshot.root.data_store_count, 1);
    assert_eq!(
        snapshot.root.data_stores[0]
            .data_state
            .as_ref()
            .and_then(|state| state.name.as_deref()),
        Some("archived")
    );
    let process = &snapshot.processes[0];
    assert_eq!(
        process.data_objects[0]
            .data_state
            .as_ref()
            .and_then(|state| state.data_state_id.as_deref()),
        Some("DataState_ObjectDraft")
    );
    assert_eq!(
        process.data_object_references[0]
            .data_state
            .as_ref()
            .and_then(|state| state.name.as_deref()),
        Some("submitted")
    );
    assert_eq!(
        process.data_store_references[0]
            .data_state
            .as_ref()
            .and_then(|state| state.name.as_deref()),
        Some("available")
    );
    assert_eq!(
        process.io_specifications[0].data_inputs[0]
            .data_state
            .as_ref()
            .and_then(|state| state.name.as_deref()),
        Some("received")
    );
    assert_eq!(
        process.io_specifications[0].data_outputs[0]
            .data_state
            .as_ref()
            .and_then(|state| state.name.as_deref()),
        Some("approved")
    );
}

#[test]
fn bpmn_linter_accepts_data_store_reference_binding_metadata() {
    let source = bpmn_fixture_source("invalid-data-store-binding.bpmn");
    let report = lint_bpmn_source(&source);

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "standard data-store bindings should import as metadata: {report:?}"
    );
    assert!(report.issues.is_empty());

    let snapshot = snapshot_bpmn_source(&source)
        .unwrap_or_else(|error| panic!("data-store binding fixture should snapshot: {error}"));
    assert_eq!(snapshot.root.data_store_count, 1);
    let process = &snapshot.processes[0];
    assert_eq!(process.data_store_reference_count, 1);
    assert_eq!(process.data_input_association_count, 1);
    assert_eq!(process.data_output_association_count, 1);
    assert_eq!(
        process.data_store_references[0]
            .data_store_reference_id
            .as_deref(),
        Some("DataStoreReference_Orders")
    );
    assert_eq!(
        process.data_store_references[0].data_store_ref.as_deref(),
        Some("DataStore_Orders")
    );
    assert_eq!(
        process.data_input_associations[0].source_refs,
        vec!["DataStoreReference_Orders".to_string()]
    );
    assert_eq!(
        process.data_output_associations[0].target_ref.as_deref(),
        Some("DataStoreReference_Orders")
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
