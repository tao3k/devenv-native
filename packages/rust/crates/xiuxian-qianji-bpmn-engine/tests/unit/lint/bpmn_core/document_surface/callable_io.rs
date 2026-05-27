use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};
use xiuxian_qianji_bpmn_engine::snapshot_bpmn_source;

#[test]
fn bpmn_linter_preserves_callable_io_metadata_surface() {
    let source = bpmn_fixture_source("metadata-callable-io.bpmn");
    let report = lint_bpmn_source(&source);

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "callable IO metadata should lint cleanly as passive metadata: {report:?}"
    );
    assert!(report.issues.is_empty());

    let snapshot = snapshot_bpmn_source(&source)
        .unwrap_or_else(|error| panic!("callable IO fixture should snapshot: {error}"));
    let process = &snapshot.processes[0];
    let global_task = &snapshot.root.global_tasks[0];
    assert_eq!(process.io_binding_count, 1);
    assert_eq!(global_task.io_specification_count, 1);
    assert_eq!(global_task.io_binding_count, 1);
    assert_eq!(
        process.io_bindings[0].operation_ref.as_deref(),
        Some("Operation_Callable")
    );
    assert_eq!(
        global_task.io_specifications[0].data_inputs[0]
            .data_id
            .as_deref(),
        Some("GlobalInput_Request")
    );
    assert_eq!(
        global_task.io_bindings[0].output_data_ref.as_deref(),
        Some("GlobalOutput_Response")
    );
}
