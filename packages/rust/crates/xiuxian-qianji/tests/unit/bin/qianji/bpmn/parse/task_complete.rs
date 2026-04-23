use super::*;

#[cfg(feature = "sqlite")]
#[test]
fn parse_bpmn_command_accepts_tasks_complete_with_sqlite_checkpoint_backend() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "tasks",
                "complete",
                "--bpmn",
                "fixtures/review.bpmn",
                "--instance-id",
                "wf_service",
                "--host-fixture",
                "fixtures/host.json",
                "--checkpoint-sqlite",
                "state.sqlite3",
            ])),
            "bpmn tasks complete parse should succeed",
        ),
        "bpmn tasks complete command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::TaskComplete(BpmnTaskCompleteCliCommand {
            bpmn_path: PathBuf::from("fixtures/review.bpmn"),
            dmn_paths: Vec::new(),
            instance_id: "wf_service".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(PathBuf::from("state.sqlite3")),
            host_fixture_path: Some(PathBuf::from("fixtures/host.json")),
            event_fixture_path: None,
        })
    );
}

#[test]
fn parse_bpmn_command_rejects_tasks_complete_without_checkpoint_backend() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "tasks",
        "complete",
        "--bpmn",
        "fixtures/review.bpmn",
        "--instance-id",
        "wf_service",
    ])) {
        Ok(command) => panic!("missing checkpoint backend should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing checkpoint backend for `bpmn tasks complete`")
    );
}
