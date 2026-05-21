use super::metadata_snapshot;
use crate::test_support::MustExt as _;

#[test]
fn bpmn_snapshot_preserves_collaboration_metadata_item_catalog() {
    let snapshot = metadata_snapshot();

    assert_eq!(
        snapshot.root.definitions_id.as_deref(),
        Some("Defs_Metadata")
    );
    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("http://www.omg.org/spec/BPMN/20100524/MODEL")
    );
    assert_eq!(snapshot.root.collaboration_count, 1);
    assert_eq!(snapshot.root.process_count, 1);
    assert_eq!(snapshot.root.item_definition_count, 1);
    assert_eq!(
        snapshot.root.item_definitions[0]
            .item_definition_id
            .as_deref(),
        Some("Item_Order")
    );
    assert_eq!(
        snapshot.root.item_definitions[0].structure_ref.as_deref(),
        Some("tns:Order")
    );
    assert_eq!(
        snapshot.root.item_definitions[0].item_kind.as_deref(),
        Some("Information")
    );
    assert_eq!(
        snapshot.root.item_definitions[0]
            .is_collection
            .map(qianji_bpmn_engine::bpmn_model_api::BpmnSnapshotFlag::get),
        Some(false)
    );
    assert_eq!(snapshot.root.message_count, 1);
    assert_eq!(
        snapshot.root.messages[0].message_id.as_deref(),
        Some("Message_1")
    );
    assert_eq!(
        snapshot.root.messages[0].item_ref.as_deref(),
        Some("Item_Order")
    );
    assert_eq!(snapshot.root.correlation_property_count, 1);
    assert_eq!(
        snapshot.root.correlation_properties[0]
            .correlation_property_id
            .as_deref(),
        Some("Correlation_Order")
    );
    assert_eq!(
        snapshot.root.correlation_properties[0].type_ref.as_deref(),
        Some("tns:OrderId")
    );
    let retrieval = &snapshot.root.correlation_properties[0].retrieval_expressions[0];
    assert_eq!(
        retrieval.retrieval_expression_id.as_deref(),
        Some("Correlation_Order_From_Message")
    );
    assert_eq!(retrieval.message_ref.as_deref(), Some("Message_1"));
    assert_eq!(retrieval.message_path.as_deref(), Some("order.id"));
    assert_eq!(snapshot.root.data_store_count, 1);
    assert_eq!(
        snapshot.root.data_stores[0].data_store_id.as_deref(),
        Some("Store_1")
    );
    assert_eq!(
        snapshot.root.data_stores[0].item_subject_ref.as_deref(),
        Some("Item_Order")
    );

    let collaboration = &snapshot.collaborations[0];
    assert_eq!(
        collaboration.collaboration_id.as_deref(),
        Some("Collaboration_1")
    );
    assert_eq!(collaboration.participants.len(), 2);
    assert_eq!(
        collaboration.participants[0].process_ref.as_deref(),
        Some("Process_1")
    );
    assert_eq!(
        collaboration.message_flows[0].source_ref.as_deref(),
        Some("Participant_A")
    );
    assert_eq!(
        collaboration.message_flows[0].message_ref.as_deref(),
        Some("Message_1")
    );
}

#[test]
fn bpmn_snapshot_preserves_collaboration_metadata_lane_and_data_metadata() {
    let snapshot = metadata_snapshot();

    let process = snapshot
        .process("Process_1")
        .must("process metadata should be indexed by id");
    assert_eq!(process.lane_set_count, 1);
    assert_eq!(process.lane_sets[0].lanes[0].flow_node_refs, ["Task_1"]);
    assert_eq!(process.data_object_count, 1);
    assert_eq!(
        process.data_objects[0].item_subject_ref.as_deref(),
        Some("Item_Order")
    );
    assert_eq!(process.data_object_reference_count, 1);
    assert_eq!(
        process.data_object_references[0].data_object_ref.as_deref(),
        Some("DataObject_1")
    );
    assert_eq!(process.data_store_reference_count, 1);
    assert_eq!(
        process.data_store_references[0].data_store_ref.as_deref(),
        Some("Store_1")
    );
    assert_eq!(process.io_specification_count, 1);
    assert_eq!(
        process.io_specifications[0].data_inputs[0]
            .data_id
            .as_deref(),
        Some("Input_1")
    );
    assert_eq!(
        process.io_specifications[0].data_inputs[0]
            .item_subject_ref
            .as_deref(),
        Some("Item_Order")
    );
    assert_eq!(process.data_input_association_count, 1);
    assert_eq!(
        process.data_input_associations[0].source_refs,
        ["DataObjectReference_1"]
    );
    assert_eq!(
        process.data_input_associations[0].target_ref.as_deref(),
        Some("Input_1")
    );
}
