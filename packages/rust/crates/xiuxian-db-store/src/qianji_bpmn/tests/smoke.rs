use super::support::{
    must_ok, must_some, open_file_store, sample_checkpoint, sample_checkpoint_with_package,
    sample_package,
};
use crate::qianji_bpmn::{
    QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY, QianjiBpmnDataRecord, QianjiBpmnDataStoreError,
};
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

#[test]
fn duckdb_workflow_state_snapshot_round_trips_checkpoint_envelope() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state.duckdb");
    let checkpoint = sample_checkpoint("wf_duckdb_state", 7, json!({ "approved": true }));

    must_ok(
        store.upsert_workflow_state(&checkpoint),
        "DuckDB workflow-state upsert should succeed",
    );

    let loaded = must_some(
        must_ok(
            store.load_workflow_state("wf_duckdb_state"),
            "DuckDB workflow-state load should succeed",
        ),
        "DuckDB workflow-state snapshot should exist",
    );

    assert_eq!(loaded, checkpoint);
    let state_record = must_some(
        must_ok(
            store.load_record("wf_duckdb_state", QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY),
            "reserved workflow-state record should load",
        ),
        "reserved workflow-state record should exist",
    );
    assert_eq!(state_record.updated_at_ms, checkpoint.state.updated_at_ms);
}

#[test]
fn duckdb_workflow_state_snapshot_batch_round_trips_checkpoint_envelopes() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-batch.duckdb");
    let package = sample_package();
    let first = sample_checkpoint_with_package(&package, "wf_batch_a", 1, json!({ "risk": "low" }));
    let second =
        sample_checkpoint_with_package(&package, "wf_batch_b", 2, json!({ "risk": "high" }));

    let upserted = must_ok(
        store.upsert_workflow_states([&first, &second]),
        "workflow-state batch should save",
    );
    assert_eq!(upserted, 2);

    let loaded_first = must_some(
        must_ok(
            store.load_workflow_state("wf_batch_a"),
            "first workflow-state batch record should load",
        ),
        "first workflow-state batch record should exist",
    );
    let loaded_second = must_some(
        must_ok(
            store.load_workflow_state("wf_batch_b"),
            "second workflow-state batch record should load",
        ),
        "second workflow-state batch record should exist",
    );
    assert_eq!(loaded_first.sequence, 1);
    assert_eq!(loaded_second.sequence, 2);

    let deleted = must_ok(
        store.delete_workflow_states(["wf_batch_a", "wf_batch_b"]),
        "workflow-state batch should delete",
    );
    assert_eq!(deleted, 2);
    let missing = must_ok(
        store.load_workflow_state("wf_batch_a"),
        "deleted workflow-state batch record should query",
    );
    assert!(missing.is_none());
}

#[test]
fn duckdb_workflow_state_append_log_loads_latest_checkpoint_snapshot() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-append-log.duckdb");
    let package = sample_package();
    let first =
        sample_checkpoint_with_package(&package, "wf_append_log", 1, json!({ "risk": "low" }));
    let second =
        sample_checkpoint_with_package(&package, "wf_append_log", 2, json!({ "risk": "high" }));

    let appended = must_ok(
        store.append_workflow_state_snapshots([&first, &second]),
        "workflow-state append log should save through appender",
    );
    assert_eq!(appended, 2);
    assert_eq!(
        must_ok(
            store.workflow_state_snapshot_count("wf_append_log"),
            "workflow-state append log count should load",
        ),
        2
    );

    let loaded = must_some(
        must_ok(
            store.load_latest_workflow_state_snapshot("wf_append_log"),
            "latest append-log checkpoint should load",
        ),
        "latest append-log checkpoint should exist",
    );
    assert_eq!(loaded.sequence, 2);
    assert_eq!(loaded.state.variables["risk"], json!("high"));

    assert!(must_ok(
        store.delete_workflow_state_snapshots("wf_append_log"),
        "workflow-state append log delete should succeed",
    ));
    assert_eq!(
        must_ok(
            store.workflow_state_snapshot_count("wf_append_log"),
            "workflow-state append log count should load after delete",
        ),
        0
    );
}

#[test]
fn duckdb_workflow_state_point_append_loads_latest_checkpoint_snapshot() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-point-append.duckdb");
    let package = sample_package();
    let newer =
        sample_checkpoint_with_package(&package, "wf_point_append", 2, json!({ "risk": "high" }));
    let older =
        sample_checkpoint_with_package(&package, "wf_point_append", 1, json!({ "risk": "low" }));

    must_ok(
        store.append_workflow_state_snapshot(&newer),
        "newer workflow-state point append should save",
    );
    must_ok(
        store.append_workflow_state_snapshot(&older),
        "stale workflow-state point append should save as history",
    );

    let loaded = must_some(
        must_ok(
            store.load_latest_workflow_state_snapshot("wf_point_append"),
            "latest point-append checkpoint should load",
        ),
        "latest point-append checkpoint should exist",
    );
    assert_eq!(loaded.sequence, 2);
    assert_eq!(loaded.state.variables["risk"], json!("high"));
    assert_eq!(
        must_ok(
            store.workflow_state_snapshot_count("wf_point_append"),
            "workflow-state point append count should load",
        ),
        2
    );
}

#[test]
fn duckdb_workflow_state_compacts_latest_checkpoint_snapshot() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-compacted-latest.duckdb");
    let package = sample_package();
    let newer = sample_checkpoint_with_package(
        &package,
        "wf_compacted_latest",
        3,
        json!({ "risk": "high" }),
    );
    let older = sample_checkpoint_with_package(
        &package,
        "wf_compacted_latest",
        2,
        json!({ "risk": "low" }),
    );

    must_ok(
        store.append_workflow_state_snapshots([&older, &newer]),
        "workflow-state append log should save compaction samples",
    );
    must_ok(
        store.compact_workflow_state_latest_snapshots(),
        "workflow-state latest compaction should succeed",
    );

    let loaded = must_some(
        must_ok(
            store.load_compacted_workflow_state_snapshot("wf_compacted_latest"),
            "compacted latest checkpoint should load",
        ),
        "compacted latest checkpoint should exist",
    );
    assert_eq!(loaded.sequence, 3);
    assert_eq!(loaded.state.variables["risk"], json!("high"));
}

#[test]
fn duckdb_workflow_state_latest_table_rejects_stale_checkpoint_snapshot() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-latest-table.duckdb");
    let package = sample_package();
    let newer =
        sample_checkpoint_with_package(&package, "wf_latest_table", 2, json!({ "risk": "high" }));
    let older =
        sample_checkpoint_with_package(&package, "wf_latest_table", 1, json!({ "risk": "low" }));

    must_ok(
        store.upsert_latest_workflow_state_snapshot(&newer),
        "newer latest-table checkpoint should save",
    );
    must_ok(
        store.upsert_latest_workflow_state_snapshot(&older),
        "stale latest-table checkpoint should be ignored",
    );

    let loaded = must_some(
        must_ok(
            store.load_latest_workflow_state_snapshot("wf_latest_table"),
            "latest-table checkpoint should load",
        ),
        "latest-table checkpoint should exist",
    );
    assert_eq!(loaded.sequence, 2);
    assert_eq!(loaded.state.variables["risk"], json!("high"));
}

#[test]
fn duckdb_workflow_state_snapshot_delete_reports_missing_after_delete() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-delete.duckdb");
    let checkpoint = sample_checkpoint("wf_duckdb_state", 8, json!({ "approved": false }));

    must_ok(
        store.upsert_workflow_state(&checkpoint),
        "DuckDB workflow-state upsert should succeed",
    );

    assert!(must_ok(
        store.delete_workflow_state("wf_duckdb_state"),
        "DuckDB workflow-state delete should succeed",
    ));
    assert!(
        must_ok(
            store.load_workflow_state("wf_duckdb_state"),
            "post-delete workflow-state load should succeed",
        )
        .is_none()
    );
}
