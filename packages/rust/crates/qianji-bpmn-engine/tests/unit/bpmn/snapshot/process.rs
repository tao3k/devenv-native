use super::snapshot_fixture;
use crate::test_support::MustExt as _;

#[test]
fn bpmn_snapshot_preserves_process_callable_metadata() {
    let snapshot = snapshot_fixture("metadata-process-callable.bpmn");

    let process = snapshot
        .process("Process_CallableMetadata")
        .must("process callable metadata should be indexed by id");
    assert_eq!(process.name.as_deref(), Some("Callable Metadata"));
    assert_eq!(process.process_type.as_deref(), Some("Public"));
    assert_eq!(
        process
            .is_closed
            .map(qianji_bpmn_engine::bpmn_model_api::BpmnSnapshotFlag::get),
        Some(true)
    );
    assert_eq!(
        process
            .is_executable
            .map(qianji_bpmn_engine::bpmn_model_api::BpmnSnapshotFlag::get),
        Some(false)
    );
    assert_eq!(
        process.definitional_collaboration_ref.as_deref(),
        Some("Collaboration_CallableMetadata")
    );
    assert_eq!(process.support_count, 1);
    assert_eq!(process.supports, ["Process_Base"]);

    assert_eq!(process.property_count, 1);
    let property = &process.properties[0];
    assert_eq!(property.property_id.as_deref(), Some("Property_Order"));
    assert_eq!(property.name.as_deref(), Some("order"));
    assert_eq!(property.item_subject_ref.as_deref(), Some("Item_Order"));

    assert_eq!(process.correlation_subscription_count, 1);
    let subscription = &process.correlation_subscriptions[0];
    assert_eq!(
        subscription.subscription_id.as_deref(),
        Some("Subscription_Order")
    );
    assert_eq!(
        subscription.correlation_key_ref.as_deref(),
        Some("CorrelationKey_Order")
    );
    assert_eq!(subscription.bindings.len(), 1);

    let binding = &subscription.bindings[0];
    assert_eq!(binding.binding_id.as_deref(), Some("Binding_Order"));
    assert_eq!(
        binding.correlation_property_ref.as_deref(),
        Some("Correlation_Order")
    );
    assert_eq!(binding.data_path.as_deref(), Some("order.id"));
    assert_eq!(
        binding.data_path_language.as_deref(),
        Some("https://example.com/jsonpath")
    );
    assert_eq!(
        binding.data_path_evaluates_to_type_ref.as_deref(),
        Some("Item_OrderId")
    );
}
