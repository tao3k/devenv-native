use super::support::{
    must_ok, must_some, open_file_store, sample_checkpoint, sample_checkpoint_with_package,
    sample_package,
};
use crate::qianji_bpmn::QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY;
use serde_json::json;
use tempfile::TempDir;

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
    assert_eq!(
        state_record.updated_at_ms.get(),
        checkpoint.state.updated_at_ms
    );
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
