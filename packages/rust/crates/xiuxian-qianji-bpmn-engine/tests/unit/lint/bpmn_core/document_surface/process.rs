use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};
use xiuxian_qianji_bpmn_engine::snapshot_bpmn_source;

#[test]
fn bpmn_linter_preserves_process_callable_metadata_surface() {
    let source = bpmn_fixture_source("metadata-process-callable.bpmn");
    let report = lint_bpmn_source(&source);

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "process callable metadata should lint cleanly as passive metadata: {report:?}"
    );
    assert!(report.issues.is_empty());

    let snapshot = snapshot_bpmn_source(&source)
        .unwrap_or_else(|error| panic!("process callable fixture should snapshot: {error}"));
    let process = &snapshot.processes[0];
    assert_eq!(process.support_count, 1);
    assert_eq!(process.property_count, 1);
    assert_eq!(process.correlation_subscription_count, 1);
    assert_eq!(process.correlation_subscriptions[0].bindings.len(), 1);
    assert_eq!(process.process_type.as_deref(), Some("Public"));
    assert_eq!(process.supports[0].as_str(), "Process_Base");
    assert_eq!(
        process.properties[0].item_subject_ref.as_deref(),
        Some("Item_Order")
    );
    assert_eq!(
        process.correlation_subscriptions[0].bindings[0]
            .data_path
            .as_deref(),
        Some("order.id")
    );
}

#[test]
fn bpmn_linter_accepts_task_property_metadata_surface() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-task-property.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "task property metadata should lint cleanly as passive metadata: {report:?}"
    );
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_reports_resource_role_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-resource-role.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_resource_role_metadata");
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(
        issue.evidence["snapshot"]["resource_roles"]["process_role_count"],
        2
    );
    assert_eq!(
        issue.evidence["snapshot"]["resource_roles"]["global_task_role_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["resource_roles"]["parameter_binding_count"],
        2
    );
    assert_eq!(
        issue.evidence["snapshot"]["resource_roles"]["assignment_expression_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["resource_roles"]["processes"][0]["resource_roles"][0]["resource_ref"],
        "Resource_Reviewer"
    );
    assert_eq!(
        issue.evidence["snapshot"]["resource_roles"]["processes"][0]["resource_roles"][1]["assignment_expression"],
        "$.review.owner"
    );
    assert_eq!(
        issue.evidence["snapshot"]["resource_roles"]["global_tasks"][0]["resource_roles"][0]["parameter_bindings"]
            [0]["expression"],
        "emea"
    );
    assert!(
        issue
            .why_it_failed
            .contains("generic assignment, scheduling, authorization")
    );
    assert_eq!(
        issue
            .structured_repair
            .as_ref()
            .and_then(|repair| repair["contract"].as_str()),
        Some("bpmn.native.resource_role.metadata_only.v1")
    );
}
