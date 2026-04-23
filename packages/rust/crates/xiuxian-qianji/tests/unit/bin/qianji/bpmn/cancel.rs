#[cfg(feature = "sqlite")]
use super::*;

#[cfg(feature = "sqlite")]
use crate::test_exports::BpmnCancelCliCommand;

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_cancel_command_deletes_waiting_sqlite_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_waiting_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("cancel.sqlite3");

    let seeded_output = must_ok(
        run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "wait_flow".to_string(),
            instance_id: "wf_cancel".to_string(),
            context_json: Some("{\"amount\":7}".to_string()),
            checkpoint_backend: Some(BpmnCliCheckpointBackend::Sqlite(sqlite_path.clone())),
            host_fixture_path: None,
            event_fixture_path: None,
        }))
        .await,
        "fresh bpmn run should seed the waiting checkpoint for cancel",
    );

    assert_eq!(seeded_output.exit_code, 0);
    assert!(seeded_output.rendered.contains("Checkpoint saved: yes"));

    let cancel_output = must_ok(
        run_bpmn_command(BpmnCliCommand::Cancel(BpmnCancelCliCommand {
            instance_id: "wf_cancel".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path.clone()),
        }))
        .await,
        "cancel command should delete the waiting sqlite checkpoint",
    );

    assert_eq!(cancel_output.exit_code, 0);
    assert!(cancel_output.rendered.starts_with("# BPMN Cancel"));
    assert!(
        cancel_output
            .rendered
            .contains("Checkpoint status: deleted")
    );
    assert!(
        cancel_output
            .rendered
            .contains("Checkpoint backend: sqlite")
    );
    assert!(
        cancel_output
            .rendered
            .contains("Lifecycle at cancel: waiting")
    );

    let store = must_some(
        must_ok(
            resolve_bpmn_checkpoint_store_with_env(
                Some(&BpmnCliCheckpointBackend::Sqlite(sqlite_path)),
                None,
            ),
            "cancelled sqlite checkpoint should resolve the checkpoint store",
        ),
        "cancelled sqlite checkpoint should expose the resolved store",
    );
    let checkpoint = must_ok(
        store.load("wf_cancel").await,
        "cancelled sqlite checkpoint should be deleted",
    );
    assert!(checkpoint.is_none());
}

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_cancel_command_renders_missing_sqlite_checkpoint_cleanly() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let sqlite_path = temp_dir.path().join("cancel-missing.sqlite3");

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::Cancel(BpmnCancelCliCommand {
            instance_id: "wf_missing_cancel".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path),
        }))
        .await,
        "cancel command should render missing checkpoint cleanly",
    );

    assert_eq!(output.exit_code, 1);
    assert!(output.rendered.starts_with("# BPMN Cancel"));
    assert!(output.rendered.contains("Checkpoint backend: sqlite"));
    assert!(output.rendered.contains("Checkpoint status: missing"));
}
