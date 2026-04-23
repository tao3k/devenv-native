use super::*;

#[cfg(feature = "sqlite")]
#[test]
fn parse_bpmn_command_accepts_events_poll_with_sqlite_checkpoint_backend() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "events",
                "poll",
                "--bpmn",
                "fixtures/wait.bpmn",
                "--instance-id",
                "wf_wait",
                "--event-fixture",
                "fixtures/events.json",
                "--checkpoint-sqlite",
                "state.sqlite3",
            ])),
            "bpmn events poll parse should succeed",
        ),
        "bpmn events poll command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::EventPoll(BpmnEventPollCliCommand {
            bpmn_path: PathBuf::from("fixtures/wait.bpmn"),
            dmn_paths: Vec::new(),
            instance_id: "wf_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(PathBuf::from("state.sqlite3")),
            host_fixture_path: None,
            event_fixture_path: Some(PathBuf::from("fixtures/events.json")),
        })
    );
}

#[test]
fn parse_bpmn_command_rejects_events_poll_without_checkpoint_backend() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "events",
        "poll",
        "--bpmn",
        "fixtures/wait.bpmn",
        "--instance-id",
        "wf_wait",
    ])) {
        Ok(command) => panic!("missing checkpoint backend should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing checkpoint backend for `bpmn events poll`")
    );
}
