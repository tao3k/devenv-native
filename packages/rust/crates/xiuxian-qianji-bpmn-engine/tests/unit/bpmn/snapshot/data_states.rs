use super::snapshot_fixture;
use crate::test_support::MustExt as _;

#[test]
fn bpmn_snapshot_preserves_data_state_metadata() {
    let snapshot = snapshot_fixture("metadata-data-state.bpmn");

    let data_store = snapshot
        .root
        .data_stores
        .iter()
        .find(|store| store.data_store_id.as_deref() == Some("DataStore_Orders"))
        .must("data store should be preserved");
    assert_eq!(
        data_store
            .data_state
            .as_ref()
            .and_then(|state| state.data_state_id.as_deref()),
        Some("DataState_StoreArchived")
    );
    assert_eq!(
        data_store
            .data_state
            .as_ref()
            .and_then(|state| state.name.as_deref()),
        Some("archived")
    );

    let process = snapshot
        .process("Process_DataState")
        .must("data-state process should be indexed by id");
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

    let io_specification = &process.io_specifications[0];
    assert_eq!(
        io_specification.data_inputs[0]
            .data_state
            .as_ref()
            .and_then(|state| state.name.as_deref()),
        Some("received")
    );
    assert_eq!(
        io_specification.data_outputs[0]
            .data_state
            .as_ref()
            .and_then(|state| state.name.as_deref()),
        Some("approved")
    );
}
