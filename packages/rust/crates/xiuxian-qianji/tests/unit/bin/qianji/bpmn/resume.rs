use super::*;

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_resume_command_completes_waiting_session_from_sqlite_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_event_wait_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("resume.sqlite3");
    let fixture_path = write_json_fixture(
        temp_dir.path().join("resume-event-fixture.json"),
        &json!({
            "event_polls": {
                "wait_message": {
                    "ready": true,
                    "data": {
                        "approved": true,
                        "source": "resume_fixture"
                    }
                }
            }
        }),
    );

    let seeded_output = must_ok(
        run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path: bpmn_path.clone(),
            dmn_paths: Vec::new(),
            process_id: "wait_flow".to_string(),
            instance_id: "wf_resume".to_string(),
            context_json: Some("{\"amount\":7}".to_string()),
            checkpoint_backend: Some(BpmnCliCheckpointBackend::Sqlite(sqlite_path.clone())),
            host_fixture_path: None,
            event_fixture_path: None,
        }))
        .await,
        "fresh bpmn run should seed the waiting checkpoint for resume",
    );

    assert_eq!(seeded_output.exit_code, 0);
    assert!(
        seeded_output
            .rendered
            .contains("Outcome: waiting_external_event")
    );
    assert!(seeded_output.rendered.contains("Checkpoint saved: yes"));

    let resumed_output = must_ok(
        run_bpmn_command(BpmnCliCommand::Resume(BpmnResumeCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            instance_id: "wf_resume".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path),
            host_fixture_path: None,
            event_fixture_path: Some(fixture_path.clone()),
        }))
        .await,
        "resume command should continue the waiting checkpoint",
    );

    assert_eq!(resumed_output.exit_code, 0);
    assert!(resumed_output.rendered.starts_with("# BPMN Resume"));
    assert!(resumed_output.rendered.contains("Outcome: completed"));
    assert!(
        resumed_output
            .rendered
            .contains("Checkpoint source: resumed")
    );
    assert!(
        resumed_output
            .rendered
            .contains(&format!("Event fixture: {}", fixture_path.display()))
    );
    assert!(resumed_output.rendered.contains("\"approved\": true"));
    assert!(
        resumed_output
            .rendered
            .contains("\"source\": \"resume_fixture\"")
    );
}

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_resume_command_renders_missing_sqlite_checkpoint_cleanly() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_event_wait_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("resume-missing.sqlite3");

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::Resume(BpmnResumeCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            instance_id: "wf_missing_resume".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path),
            host_fixture_path: None,
            event_fixture_path: None,
        }))
        .await,
        "resume command should render missing checkpoint cleanly",
    );

    assert_eq!(output.exit_code, 1);
    assert!(output.rendered.starts_with("# BPMN Resume"));
    assert!(output.rendered.contains("Checkpoint backend: sqlite"));
    assert!(output.rendered.contains("Checkpoint status: missing"));
}
