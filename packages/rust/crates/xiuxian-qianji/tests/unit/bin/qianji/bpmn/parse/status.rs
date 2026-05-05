#[cfg(feature = "duckdb")]
use super::{
    BpmnCliCheckpointBackend, BpmnCliCommand, BpmnStatusCliCommand, must_ok, must_some,
    parse_bpmn_command, to_args,
};
#[cfg(not(feature = "duckdb"))]
use super::{parse_bpmn_command, to_args};

#[cfg(not(feature = "duckdb"))]
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

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_defaults_status_to_local_duckdb_checkpoint_backend() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "status",
                "--instance-id",
                "wf_wait",
            ])),
            "bpmn status parse should default local workflow-state store",
        ),
        "bpmn status command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Status(BpmnStatusCliCommand {
            instance_id: "wf_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
            bpmn_path: None,
            dmn_paths: Vec::new(),
        })
    );
}

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_accepts_status_bpmn_context() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "status",
                "--instance-id",
                "wf_wait",
                "--bpmn",
                "workflow.bpmn",
                "--dmn",
                "rules.dmn",
            ])),
            "bpmn status parse should accept optional graph context paths",
        ),
        "bpmn status command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Status(BpmnStatusCliCommand {
            instance_id: "wf_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
            bpmn_path: Some("workflow.bpmn".into()),
            dmn_paths: vec!["rules.dmn".into()],
        })
    );
}
