use super::*;

#[cfg(feature = "sqlite")]
#[test]
fn parse_bpmn_command_accepts_resume_with_sqlite_checkpoint_backend() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "resume",
                "--bpmn",
                "fixtures/wait.bpmn",
                "--dmn",
                "fixtures/wait.dmn",
                "--instance-id",
                "wf_wait",
                "--checkpoint-sqlite",
                "state.sqlite3",
                "--event-fixture",
                "fixtures/events.json",
            ])),
            "bpmn resume parse should succeed",
        ),
        "bpmn resume command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Resume(BpmnResumeCliCommand {
            bpmn_path: PathBuf::from("fixtures/wait.bpmn"),
            dmn_paths: vec![PathBuf::from("fixtures/wait.dmn")],
            instance_id: "wf_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(PathBuf::from("state.sqlite3")),
            host_fixture_path: None,
            event_fixture_path: Some(PathBuf::from("fixtures/events.json")),
        })
    );
}

#[test]
fn parse_bpmn_command_rejects_resume_without_checkpoint_backend() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "resume",
        "--bpmn",
        "fixtures/wait.bpmn",
        "--instance-id",
        "wf_wait",
    ])) {
        Ok(command) => panic!("missing resume checkpoint backend should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing checkpoint backend for `bpmn resume`")
    );
}
