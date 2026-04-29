use super::*;

#[test]
fn bpmn_linter_reports_process_callable_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-process-callable.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_collaboration_surface");
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["support_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["property_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["correlation_subscription_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["correlation_binding_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["processes"][0]["process_type"],
        "Public"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["processes"][0]["supports"][0],
        "Process_Base"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["processes"][0]["properties"][0]["item_subject_ref"],
        "Item_Order"
    );
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["processes"][0]["correlation_subscriptions"]
            [0]["bindings"][0]["data_path"],
        "order.id"
    );
}

#[test]
fn bpmn_linter_reports_resource_role_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-resource-role.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_collaboration_surface");
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
}
