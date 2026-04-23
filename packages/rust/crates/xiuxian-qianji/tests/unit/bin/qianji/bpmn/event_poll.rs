#[cfg(feature = "sqlite")]
use super::*;

#[cfg(feature = "sqlite")]
use crate::test_exports::BpmnEventPollCliCommand;

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_events_poll_command_resolves_waiting_sqlite_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_event_wait_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("event-poll.sqlite3");
    let fixture_path = write_json_fixture(
        temp_dir.path().join("event-poll-fixture.json"),
        &json!({
            "event_polls": {
                "wait_message": {
                    "ready": true,
                    "data": {
                        "approved": true,
                        "source": "event_poll_command"
                    }
                }
            }
        }),
    );

    let seeded_output = must_ok(
        run_bpmn_command(BpmnCliCommand::Start(BpmnStartCliCommand {
            bpmn_path: bpmn_path.clone(),
            dmn_paths: Vec::new(),
            process_id: "wait_flow".to_string(),
            instance_id: "wf_event_poll".to_string(),
            context_json: Some("{\"amount\":7}".to_string()),
            checkpoint_backend: Some(BpmnCliCheckpointBackend::Sqlite(sqlite_path.clone())),
            host_fixture_path: None,
            event_fixture_path: None,
        }))
        .await,
        "bpmn start should seed one waiting sqlite checkpoint for event polling",
    );

    assert_eq!(seeded_output.exit_code, 0);
    assert!(
        seeded_output
            .rendered
            .contains("Outcome: waiting_external_event")
    );
    assert!(seeded_output.rendered.contains("Checkpoint saved: yes"));

    let poll_output = must_ok(
        run_bpmn_command(BpmnCliCommand::EventPoll(BpmnEventPollCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            instance_id: "wf_event_poll".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path),
            host_fixture_path: None,
            event_fixture_path: Some(fixture_path.clone()),
        }))
        .await,
        "bpmn events poll should resolve the waiting sqlite checkpoint",
    );

    assert_eq!(poll_output.exit_code, 0);
    assert!(poll_output.rendered.starts_with("# BPMN Event Poll"));
    assert!(poll_output.rendered.contains("Outcome: completed"));
    assert!(poll_output.rendered.contains("Checkpoint source: resumed"));
    assert!(
        poll_output
            .rendered
            .contains(&format!("Event fixture: {}", fixture_path.display()))
    );
    assert!(poll_output.rendered.contains("\"approved\": true"));
    assert!(
        poll_output
            .rendered
            .contains("\"source\": \"event_poll_command\"")
    );
}

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_events_poll_command_renders_missing_sqlite_checkpoint_cleanly() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_event_wait_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("event-poll-missing.sqlite3");

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::EventPoll(BpmnEventPollCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            instance_id: "wf_missing_event_poll".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path),
            host_fixture_path: None,
            event_fixture_path: None,
        }))
        .await,
        "bpmn events poll should render missing checkpoint cleanly",
    );

    assert_eq!(output.exit_code, 1);
    assert!(output.rendered.starts_with("# BPMN Event Poll"));
    assert!(output.rendered.contains("Checkpoint backend: sqlite"));
    assert!(output.rendered.contains("Checkpoint status: missing"));
}
