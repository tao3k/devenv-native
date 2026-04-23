use super::*;
use qianji_bpmn_engine::{
    BPMN_CHECKPOINT_FORMAT_VERSION, BpmnEngineError, delete_checkpoint_sql, load_checkpoint_sql,
    save_checkpoint_sql,
};
use serde_json::json;
use tempfile::TempDir;

#[test]
fn checkpoint_sql_round_trip_persists_latest_state() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let database_path = temp_dir.path().join("checkpoints.sqlite");
    let first = sample_checkpoint_with_sequence(1, json!({ "amount": 7 }));
    let newer = sample_checkpoint_with_sequence(2, json!({ "amount": 9 }));

    save_checkpoint_sql(&first, &database_path)
        .unwrap_or_else(|error| panic!("initial SQL checkpoint save should succeed: {error}"));
    save_checkpoint_sql(&newer, &database_path)
        .unwrap_or_else(|error| panic!("newer SQL checkpoint save should succeed: {error}"));

    let loaded = load_checkpoint_sql("wf_checkpoint", &database_path)
        .unwrap_or_else(|error| panic!("SQL checkpoint load should succeed: {error}"))
        .unwrap_or_else(|| panic!("SQL checkpoint should exist after save"));
    assert_eq!(loaded.version, BPMN_CHECKPOINT_FORMAT_VERSION);
    assert_eq!(loaded.sequence, newer.sequence);
    assert_eq!(loaded.state, newer.state);
}

#[test]
fn checkpoint_sql_rejects_stale_sequences() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let database_path = temp_dir.path().join("checkpoints.sqlite");
    let current = sample_checkpoint_with_sequence(5, json!({ "amount": 11 }));
    let equal = sample_checkpoint_with_sequence(5, json!({ "amount": 13 }));
    let older = sample_checkpoint_with_sequence(4, json!({ "amount": 3 }));

    save_checkpoint_sql(&current, &database_path)
        .unwrap_or_else(|error| panic!("current SQL checkpoint save should succeed: {error}"));

    let equal_error = save_checkpoint_sql(&equal, &database_path)
        .must_err("equal SQL checkpoint sequence should be rejected");
    assert_eq!(
        equal_error,
        BpmnEngineError::StaleCheckpointWrite {
            instance_id: "wf_checkpoint".to_string(),
            attempted_sequence: 5,
            stored_sequence: 5,
        }
    );

    let older_error = save_checkpoint_sql(&older, &database_path)
        .must_err("older SQL checkpoint sequence should be rejected");
    assert_eq!(
        older_error,
        BpmnEngineError::StaleCheckpointWrite {
            instance_id: "wf_checkpoint".to_string(),
            attempted_sequence: 4,
            stored_sequence: 5,
        }
    );
}

#[test]
fn checkpoint_sql_directory_path_returns_storage_error() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let checkpoint = sample_checkpoint();
    let error = save_checkpoint_sql(&checkpoint, temp_dir.path())
        .must_err("directory path should fail for SQL checkpoint storage");

    match error {
        BpmnEngineError::CheckpointStorage { operation, .. } => {
            assert_eq!(operation, "save_checkpoint_sql_open");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn checkpoint_sql_delete_removes_persisted_state() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let database_path = temp_dir.path().join("checkpoints.sqlite");
    let checkpoint = sample_checkpoint_with_sequence(3, json!({ "amount": 17 }));

    save_checkpoint_sql(&checkpoint, &database_path)
        .unwrap_or_else(|error| panic!("checkpoint save should succeed before delete: {error}"));
    delete_checkpoint_sql(checkpoint.state.instance_id.as_ref(), &database_path)
        .unwrap_or_else(|error| panic!("checkpoint delete should succeed: {error}"));

    let loaded = load_checkpoint_sql(checkpoint.state.instance_id.as_ref(), &database_path)
        .unwrap_or_else(|error| panic!("checkpoint load should succeed after delete: {error}"));
    assert!(loaded.is_none());
}
