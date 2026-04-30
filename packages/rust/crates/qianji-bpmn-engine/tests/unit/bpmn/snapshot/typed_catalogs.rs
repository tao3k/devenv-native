use super::snapshot_fixture;

#[test]
fn bpmn_snapshot_preserves_interface_operation_metadata_catalogs() {
    let snapshot = snapshot_fixture("metadata-interface-operation-catalog.bpmn");

    assert_eq!(snapshot.root.interface_count, 1);
    let interface = &snapshot.root.interfaces[0];
    assert_eq!(interface.interface_id.as_deref(), Some("Interface_Order"));
    assert_eq!(interface.name.as_deref(), Some("Order Interface"));
    assert_eq!(
        interface.implementation_ref.as_deref(),
        Some("tns:OrderPort")
    );
    assert_eq!(interface.operations.len(), 1);

    let operation = &interface.operations[0];
    assert_eq!(operation.operation_id.as_deref(), Some("Operation_Submit"));
    assert_eq!(operation.name.as_deref(), Some("Submit Order"));
    assert_eq!(
        operation.implementation_ref.as_deref(),
        Some("tns:submitOrder")
    );
    assert_eq!(operation.in_message_ref.as_deref(), Some("Message_Request"));
    assert_eq!(
        operation.out_message_ref.as_deref(),
        Some("Message_Response")
    );
    assert_eq!(operation.error_refs, ["Service_Error"]);
}

#[test]
fn bpmn_snapshot_preserves_resource_metadata_catalogs() {
    let snapshot = snapshot_fixture("metadata-resource-catalog.bpmn");

    assert_eq!(snapshot.root.resource_count, 1);
    let resource = &snapshot.root.resources[0];
    assert_eq!(resource.resource_id.as_deref(), Some("Resource_Reviewer"));
    assert_eq!(resource.name.as_deref(), Some("Reviewer"));
    assert_eq!(resource.resource_parameters.len(), 2);

    let region = &resource.resource_parameters[0];
    assert_eq!(
        region.resource_parameter_id.as_deref(),
        Some("ResourceParam_Region")
    );
    assert_eq!(region.name.as_deref(), Some("region"));
    assert_eq!(region.type_ref.as_deref(), Some("Item_Region"));
    assert_eq!(region.is_required, Some(true));

    let level = &resource.resource_parameters[1];
    assert_eq!(
        level.resource_parameter_id.as_deref(),
        Some("ResourceParam_Level")
    );
    assert_eq!(level.name.as_deref(), Some("level"));
    assert_eq!(level.type_ref.as_deref(), Some("Item_Level"));
    assert_eq!(level.is_required, Some(false));
}

#[test]
fn bpmn_snapshot_preserves_category_metadata_catalogs() {
    let snapshot = snapshot_fixture("metadata-category-catalog.bpmn");

    assert_eq!(snapshot.root.category_count, 1);
    let category = &snapshot.root.categories[0];
    assert_eq!(category.category_id.as_deref(), Some("Category_Risk"));
    assert_eq!(category.name.as_deref(), Some("Risk"));
    assert_eq!(category.category_values.len(), 2);

    let high_risk = &category.category_values[0];
    assert_eq!(
        high_risk.category_value_id.as_deref(),
        Some("CategoryValue_HighRisk")
    );
    assert_eq!(high_risk.value.as_deref(), Some("high-risk"));

    let manual_review = &category.category_values[1];
    assert_eq!(
        manual_review.category_value_id.as_deref(),
        Some("CategoryValue_ManualReview")
    );
    assert_eq!(manual_review.value.as_deref(), Some("manual-review"));
}

#[test]
fn bpmn_snapshot_preserves_global_task_metadata_catalogs() {
    let snapshot = snapshot_fixture("metadata-global-task-catalog.bpmn");

    assert_eq!(snapshot.root.global_task_count, 5);
    assert_eq!(
        snapshot.root.global_tasks[0].task_id.as_deref(),
        Some("GlobalTask_Generic")
    );
    assert_eq!(
        snapshot.root.global_tasks[0].supported_interface_refs,
        ["Interface_Work"]
    );
    assert_eq!(
        snapshot.root.global_tasks[1].task_kind,
        "globalBusinessRuleTask"
    );
    assert_eq!(
        snapshot.root.global_tasks[1].implementation.as_deref(),
        Some("##unspecified")
    );
    assert_eq!(snapshot.root.global_tasks[2].task_kind, "globalManualTask");
    assert_eq!(snapshot.root.global_tasks[3].task_kind, "globalScriptTask");
    assert_eq!(
        snapshot.root.global_tasks[3].script_language.as_deref(),
        Some("text/javascript")
    );
    assert_eq!(
        snapshot.root.global_tasks[3].script.as_deref(),
        Some("return input.approved === true;")
    );
    assert_eq!(snapshot.root.global_tasks[4].task_kind, "globalUserTask");
    assert_eq!(
        snapshot.root.global_tasks[4].implementation.as_deref(),
        Some("##WebService")
    );
}
