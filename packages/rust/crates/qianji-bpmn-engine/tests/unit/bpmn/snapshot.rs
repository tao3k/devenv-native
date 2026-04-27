use super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnSourceFile, snapshot_bpmn_source};

#[test]
fn bpmn_snapshot_preserves_collaboration_lane_and_data_metadata() {
    let snapshot = snapshot_bpmn_source(&fixture_source("metadata-collaboration-lane-data.bpmn"))
        .must("metadata-only BPMN source should still produce a document snapshot");

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
    assert_eq!(snapshot.root.data_store_count, 1);
    assert_eq!(
        snapshot.root.data_stores[0].data_store_id.as_deref(),
        Some("Store_1")
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

    let process = snapshot
        .process("Process_1")
        .must("process metadata should be indexed by id");
    assert_eq!(process.lane_set_count, 1);
    assert_eq!(process.lane_sets[0].lanes[0].flow_node_refs, ["Task_1"]);
    assert_eq!(process.data_object_count, 1);
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

#[test]
fn bpmn_snapshot_reports_invalid_xml_as_bpmn_xml_error() {
    let source = BpmnSourceFile::new("broken.bpmn", "<definitions><process></definitions>");

    let error = snapshot_bpmn_source(&source).must_err("invalid XML should be rejected");

    let BpmnEngineError::InvalidXml {
        source_id, offset, ..
    } = error
    else {
        panic!("invalid XML should return InvalidXml");
    };
    assert_eq!(source_id, "broken.bpmn");
    assert!(
        offset.is_some(),
        "XML reader should report an error byte offset"
    );
}
