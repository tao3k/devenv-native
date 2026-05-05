use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_callable_io_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-callable-io.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_collaboration_surface");
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["process_io_binding_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["global_task_io_specification_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["global_task_io_binding_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["processes"][0]["io_bindings"][0]["operation_ref"],
        "Operation_Callable"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["global_tasks"][0]["io_specifications"][0]["data_inputs"]
            [0]["data_id"],
        "GlobalInput_Request"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["global_tasks"][0]["io_bindings"][0]["output_data_ref"],
        "GlobalOutput_Response"
    );
}
