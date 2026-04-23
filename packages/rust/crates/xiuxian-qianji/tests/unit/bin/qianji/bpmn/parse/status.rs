use super::*;

#[cfg(feature = "sqlite")]
use crate::test_exports::BpmnStatusCliCommand;

#[cfg(feature = "sqlite")]
#[test]
fn parse_bpmn_command_accepts_status_with_sqlite_checkpoint_backend() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "status",
                "--instance-id",
                "wf_wait",
                "--checkpoint-sqlite",
                "state.sqlite3",
            ])),
            "bpmn status parse should succeed",
        ),
        "bpmn status command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Status(BpmnStatusCliCommand {
            instance_id: "wf_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(PathBuf::from("state.sqlite3")),
        })
    );
}

#[test]
fn parse_bpmn_command_rejects_status_without_checkpoint_backend() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "status",
        "--instance-id",
        "wf_wait",
    ])) {
        Ok(command) => panic!("missing status checkpoint backend should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing checkpoint backend for `bpmn status`")
    );
}
