use super::snapshot_fixture;
use crate::test_support::MustExt as _;

#[test]
fn bpmn_snapshot_preserves_resource_role_metadata() {
    let snapshot = snapshot_fixture("metadata-resource-role.bpmn");

    let global_task = &snapshot.root.global_tasks[0];
    assert_eq!(
        global_task.task_id.as_deref(),
        Some("GlobalTask_ResourceRole")
    );
    assert_eq!(global_task.resource_role_count, 1);
    let global_role = &global_task.resource_roles[0];
    assert_eq!(global_role.role_kind, "humanPerformer");
    assert_eq!(global_role.role_id.as_deref(), Some("GlobalRole_Reviewer"));
    assert_eq!(global_role.name.as_deref(), Some("global_reviewer"));
    assert_eq!(
        global_role.resource_ref.as_deref(),
        Some("Resource_Reviewer")
    );
    assert_eq!(global_role.parameter_bindings.len(), 1);
    assert_eq!(
        global_role.parameter_bindings[0].parameter_ref.as_deref(),
        Some("ResourceParam_Region")
    );
    assert_eq!(
        global_role.parameter_bindings[0].expression.as_deref(),
        Some("emea")
    );
    assert_eq!(
        global_role.parameter_bindings[0]
            .expression_evaluates_to_type_ref
            .as_deref(),
        Some("Item_Region")
    );

    let process = snapshot
        .process("Process_ResourceRoleMetadata")
        .must("process resource-role metadata should be indexed by id");
    assert_eq!(process.resource_role_count, 2);

    let process_role = &process.resource_roles[0];
    assert_eq!(process_role.role_kind, "resourceRole");
    assert_eq!(
        process_role.role_id.as_deref(),
        Some("ProcessRole_Reviewer")
    );
    assert_eq!(
        process_role.resource_ref.as_deref(),
        Some("Resource_Reviewer")
    );
    assert_eq!(
        process_role.parameter_bindings[0].binding_id.as_deref(),
        Some("ProcessRole_Level")
    );
    assert_eq!(
        process_role.parameter_bindings[0].expression.as_deref(),
        Some("senior")
    );

    let assignment_role = &process.resource_roles[1];
    assert_eq!(assignment_role.role_kind, "performer");
    assert_eq!(
        assignment_role.assignment_expression_id.as_deref(),
        Some("ProcessRole_Assignment")
    );
    assert_eq!(
        assignment_role.assignment_expression.as_deref(),
        Some("$.review.owner")
    );
    assert_eq!(
        assignment_role.assignment_expression_language.as_deref(),
        Some("https://example.com/jsonpath")
    );
    assert_eq!(
        assignment_role
            .assignment_expression_evaluates_to_type_ref
            .as_deref(),
        Some("Item_Level")
    );
}
