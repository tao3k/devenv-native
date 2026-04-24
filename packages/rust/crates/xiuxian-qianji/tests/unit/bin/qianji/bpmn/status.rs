#[cfg(feature = "sqlite")]
use super::*;

#[cfg(feature = "sqlite")]
use crate::test_exports::BpmnStatusCliCommand;

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_status_command_renders_waiting_sqlite_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_waiting_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("bpmn-status.sqlite3");

    let run_output = must_ok(
        run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "wait_flow".to_string(),
            instance_id: "wf_status_wait".to_string(),
            context_json: Some("{}".to_string()),
            checkpoint_backend: Some(BpmnCliCheckpointBackend::Sqlite(sqlite_path.clone())),
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
        }))
        .await,
        "waiting bpmn run should save one sqlite checkpoint for status",
    );

    assert_eq!(run_output.exit_code, 0);
    assert!(run_output.rendered.contains("Checkpoint saved: yes"));

    let status_output = must_ok(
        run_bpmn_command(BpmnCliCommand::Status(BpmnStatusCliCommand {
            instance_id: "wf_status_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path),
        }))
        .await,
        "bpmn status should load one waiting sqlite checkpoint",
    );

    assert_eq!(status_output.exit_code, 0);
    assert!(status_output.rendered.starts_with("# BPMN Status"));
    assert!(status_output.rendered.contains("Checkpoint status: loaded"));
    assert!(status_output.rendered.contains("Lifecycle: waiting"));
    assert!(
        status_output
            .rendered
            .contains("Checkpoint backend: sqlite")
    );
    assert!(status_output.rendered.contains("Process: wait_flow"));
    assert!(status_output.rendered.contains("Wait registrations: 1"));
    assert!(status_output.rendered.contains("kind=external_event"));
}

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_status_command_renders_missing_sqlite_checkpoint_cleanly() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let sqlite_path = temp_dir.path().join("missing.sqlite3");

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::Status(BpmnStatusCliCommand {
            instance_id: "wf_missing".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path),
        }))
        .await,
        "bpmn status should render a missing checkpoint report",
    );

    assert_eq!(output.exit_code, 1);
    assert!(output.rendered.starts_with("# BPMN Status"));
    assert!(output.rendered.contains("Instance: wf_missing"));
    assert!(output.rendered.contains("Checkpoint status: missing"));
}
