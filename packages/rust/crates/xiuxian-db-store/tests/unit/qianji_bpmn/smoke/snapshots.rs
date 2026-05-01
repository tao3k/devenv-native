use super::super::support::{
    must_ok, must_some, open_file_store, sample_checkpoint_with_package, sample_package,
};
use serde_json::json;
use tempfile::TempDir;

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
