use super::support::{must_ok, must_some, open_file_store};
use crate::qianji_bpmn::{QianjiBpmnDataRecord, QianjiBpmnDataStoreError};
use serde_json::json;
use tempfile::TempDir;

#[test]
fn duckdb_workflow_data_store_round_trips_json_records() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-data.duckdb");
    let record = QianjiBpmnDataRecord::new(
        "wf_duckdb",
        "host_work/result",
        json!({ "status": "completed", "answer": 42 }),
        1_760_000_000_001,
    );

    must_ok(
        store.upsert_record(&record),
        "DuckDB record upsert should succeed",
    );
    let loaded = must_some(
        must_ok(
            store.load_record("wf_duckdb", "host_work/result"),
            "DuckDB record load should succeed",
        ),
        "DuckDB record should exist after upsert",
    );

    assert_eq!(loaded, record);
}

#[test]
fn duckdb_workflow_data_store_overwrites_existing_record_key() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
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

    must_ok(
        store.upsert_record(&first),
        "first DuckDB record upsert should succeed",
    );
    must_ok(
        store.upsert_record(&newer),
        "newer DuckDB record upsert should succeed",
    );

    let loaded = must_some(
        must_ok(
            store.load_record("wf_duckdb", "dmn/outcome"),
            "DuckDB overwritten record load should succeed",
        ),
        "DuckDB overwritten record should exist",
    );
    assert_eq!(loaded, newer);
}

#[test]
fn duckdb_workflow_data_store_reports_missing_and_delete() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-delete.duckdb");
    let record = QianjiBpmnDataRecord::new(
        "wf_duckdb",
        "dataset/ref",
        json!({ "dataset_name": "features" }),
        1_760_000_000_003,
    );

    assert!(
        must_ok(
            store.load_record("wf_duckdb", "dataset/ref"),
            "missing DuckDB lookup should succeed",
        )
        .is_none()
    );

    must_ok(
        store.upsert_record(&record),
        "DuckDB record upsert before delete should succeed",
    );
    assert!(must_ok(
        store.delete_record("wf_duckdb", "dataset/ref"),
        "DuckDB record delete should succeed",
    ));
    assert!(
        must_ok(
            store.load_record("wf_duckdb", "dataset/ref"),
            "post-delete DuckDB lookup should succeed",
        )
        .is_none()
    );
}

#[test]
fn duckdb_workflow_data_store_rejects_blank_record_keys() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
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
