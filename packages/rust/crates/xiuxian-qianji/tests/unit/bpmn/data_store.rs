use crate::{
    QianjiBpmnDataRecord, QianjiBpmnDataStoreError, QianjiBpmnDuckDbDataStore,
    QianjiBpmnDuckDbDataStoreConfig,
};
use serde_json::json;
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

    let error = store
        .upsert_record(&record)
        .expect_err("blank record key should be rejected");

    assert_eq!(
        error,
        QianjiBpmnDataStoreError::BlankField {
            field: "record_key"
        }
    );
}

fn open_file_store(temp_dir: &TempDir, file_name: &str) -> QianjiBpmnDuckDbDataStore {
    QianjiBpmnDuckDbDataStore::open(QianjiBpmnDuckDbDataStoreConfig::file(
        temp_dir.path().join(file_name),
    ))
    .unwrap_or_else(|error| panic!("DuckDB workflow data store should open: {error}"))
}
