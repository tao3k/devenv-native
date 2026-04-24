use crate::{
    QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY, QianjiBpmnCheckpointStore, QianjiBpmnDataRecord,
    QianjiBpmnDataStoreError, QianjiBpmnDuckDbDataStore, QianjiBpmnDuckDbDataStoreConfig,
};
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnEdgeSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec,
    BpmnPackage, BpmnProcessSpec, ProcessKey, create_instance,
};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn duckdb_workflow_data_store_round_trips_json_records() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let store = open_file_store(&temp_dir, "workflow-data.duckdb");
    let record = QianjiBpmnDataRecord::new(
        "wf_duckdb",
        "host_work/result",
        json!({ "status": "completed", "answer": 42 }),
        1_760_000_000_001,
    );

    store
        .upsert_record(&record)
        .unwrap_or_else(|error| panic!("DuckDB record upsert should succeed: {error}"));
    let loaded = store
        .load_record("wf_duckdb", "host_work/result")
        .unwrap_or_else(|error| panic!("DuckDB record load should succeed: {error}"))
        .unwrap_or_else(|| panic!("DuckDB record should exist after upsert"));

    assert_eq!(loaded, record);
}

#[test]
fn duckdb_workflow_data_store_overwrites_existing_record_key() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let store = open_file_store(&temp_dir, "workflow-overwrite.duckdb");
    let first = QianjiBpmnDataRecord::new(
        "wf_duckdb",
        "dmn/outcome",
        json!({ "risk": "low" }),
        1_760_000_000_001,
    );
    let newer = QianjiBpmnDataRecord::new(
        "wf_duckdb",
        "dmn/outcome",
        json!({ "risk": "high" }),
        1_760_000_000_002,
    );

    store
        .upsert_record(&first)
        .unwrap_or_else(|error| panic!("first DuckDB record upsert should succeed: {error}"));
    store
        .upsert_record(&newer)
        .unwrap_or_else(|error| panic!("newer DuckDB record upsert should succeed: {error}"));

    let loaded = store
        .load_record("wf_duckdb", "dmn/outcome")
        .unwrap_or_else(|error| panic!("DuckDB overwritten record load should succeed: {error}"))
        .unwrap_or_else(|| panic!("DuckDB overwritten record should exist"));
    assert_eq!(loaded, newer);
}

#[test]
fn duckdb_workflow_data_store_reports_missing_and_delete() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let store = open_file_store(&temp_dir, "workflow-delete.duckdb");
    let record = QianjiBpmnDataRecord::new(
        "wf_duckdb",
        "dataset/ref",
        json!({ "dataset_name": "features" }),
        1_760_000_000_003,
    );

    assert!(
        store
            .load_record("wf_duckdb", "dataset/ref")
            .unwrap_or_else(|error| panic!("missing DuckDB lookup should succeed: {error}"))
            .is_none()
    );

    store.upsert_record(&record).unwrap_or_else(|error| {
        panic!("DuckDB record upsert before delete should succeed: {error}")
    });
    assert!(
        store
            .delete_record("wf_duckdb", "dataset/ref")
            .unwrap_or_else(|error| panic!("DuckDB record delete should succeed: {error}"))
    );
    assert!(
        store
            .load_record("wf_duckdb", "dataset/ref")
            .unwrap_or_else(|error| panic!("post-delete DuckDB lookup should succeed: {error}"))
            .is_none()
    );
}

#[test]
fn duckdb_workflow_data_store_rejects_blank_record_keys() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let store = open_file_store(&temp_dir, "workflow-validation.duckdb");
    let record = QianjiBpmnDataRecord::new("wf_duckdb", " ", json!({}), 1);

    let error = match store.upsert_record(&record) {
        Ok(()) => panic!("blank record key should be rejected"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        QianjiBpmnDataStoreError::BlankField {
            field: "record_key"
        }
    );
}

#[test]
fn duckdb_workflow_state_snapshot_round_trips_checkpoint_envelope() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let store = open_file_store(&temp_dir, "workflow-state.duckdb");
    let checkpoint = sample_checkpoint("wf_duckdb_state", 7, json!({ "approved": true }));

    store
        .upsert_workflow_state(&checkpoint)
        .unwrap_or_else(|error| panic!("DuckDB workflow-state upsert should succeed: {error}"));

    let loaded = store
        .load_workflow_state("wf_duckdb_state")
        .unwrap_or_else(|error| panic!("DuckDB workflow-state load should succeed: {error}"))
        .unwrap_or_else(|| panic!("DuckDB workflow-state snapshot should exist"));

    assert_eq!(loaded, checkpoint);
    let state_record = store
        .load_record("wf_duckdb_state", QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY)
        .unwrap_or_else(|error| panic!("reserved workflow-state record should load: {error}"))
        .unwrap_or_else(|| panic!("reserved workflow-state record should exist"));
    assert_eq!(state_record.updated_at_ms, checkpoint.state.updated_at_ms);
}

#[test]
fn duckdb_workflow_state_snapshot_delete_reports_missing_after_delete() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let store = open_file_store(&temp_dir, "workflow-state-delete.duckdb");
    let checkpoint = sample_checkpoint("wf_duckdb_state", 8, json!({ "approved": false }));

    store
        .upsert_workflow_state(&checkpoint)
        .unwrap_or_else(|error| panic!("DuckDB workflow-state upsert should succeed: {error}"));

    assert!(
        store
            .delete_workflow_state("wf_duckdb_state")
            .unwrap_or_else(|error| panic!("DuckDB workflow-state delete should succeed: {error}"))
    );
    assert!(
        store
            .load_workflow_state("wf_duckdb_state")
            .unwrap_or_else(|error| panic!(
                "post-delete workflow-state load should succeed: {error}"
            ))
            .is_none()
    );
}

#[tokio::test]
async fn duckdb_checkpoint_store_facade_round_trips_local_workflow_state() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let store = QianjiBpmnCheckpointStore::duckdb(temp_dir.path().join("facade-state.duckdb"));
    let checkpoint = sample_checkpoint("wf_duckdb_facade", 9, json!({ "score": 99 }));

    store
        .save(&checkpoint)
        .await
        .unwrap_or_else(|error| panic!("DuckDB facade save should succeed: {error}"));

    let loaded = store
        .load("wf_duckdb_facade")
        .await
        .unwrap_or_else(|error| panic!("DuckDB facade load should succeed: {error}"))
        .unwrap_or_else(|| panic!("DuckDB facade state should exist"));
    assert_eq!(loaded, checkpoint);

    store
        .delete("wf_duckdb_facade")
        .await
        .unwrap_or_else(|error| panic!("DuckDB facade delete should succeed: {error}"));
    assert!(
        store
            .load("wf_duckdb_facade")
            .await
            .unwrap_or_else(|error| panic!(
                "post-delete DuckDB facade load should succeed: {error}"
            ))
            .is_none()
    );
}

fn open_file_store(temp_dir: &TempDir, file_name: &str) -> QianjiBpmnDuckDbDataStore {
    QianjiBpmnDuckDbDataStore::open(QianjiBpmnDuckDbDataStoreConfig::file(
        temp_dir.path().join(file_name),
    ))
    .unwrap_or_else(|error| panic!("DuckDB workflow data store should open: {error}"))
}

fn sample_checkpoint(
    instance_id: &str,
    sequence: u64,
    variables: serde_json::Value,
) -> BpmnCheckpointEnvelope {
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_duckdb", "approve", "digest_duckdb"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "end", BpmnNodeKind::EndEvent),
        ],
        vec![BpmnEdgeSpec::new(0, 1, None::<&str>)],
        Vec::new(),
    );
    let package = Arc::new(BpmnPackage::new("pkg_duckdb", vec![process]));
    let state = create_instance(
        Arc::clone(&package),
        "approve",
        BpmnInstanceInit::new(instance_id, variables, 1_760_000_000_004),
    )
    .unwrap_or_else(|error| panic!("known process should create an instance: {error}"));
    let mut state = state;
    state.sequence = sequence;
    state.updated_at_ms = 1_760_000_000_004 + sequence;
    BpmnCheckpointEnvelope::from_state(state)
}
