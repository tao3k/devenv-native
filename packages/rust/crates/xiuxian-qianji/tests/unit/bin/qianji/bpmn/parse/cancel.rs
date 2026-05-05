#[cfg(feature = "duckdb")]
use super::{
    BpmnCancelCliCommand, BpmnCliCheckpointBackend, BpmnCliCommand, must_ok, must_some,
    parse_bpmn_command, to_args,
};
#[cfg(not(feature = "duckdb"))]
use super::{parse_bpmn_command, to_args};

#[cfg(not(feature = "duckdb"))]
#[test]
fn parse_bpmn_command_rejects_cancel_without_checkpoint_backend() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "cancel",
        "--instance-id",
        "wf_wait",
    ])) {
        Ok(command) => panic!("missing cancel checkpoint backend should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing checkpoint backend for `bpmn cancel`")
    );
}

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_defaults_cancel_to_local_duckdb_checkpoint_backend() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "cancel",
                "--instance-id",
                "wf_wait",
            ])),
            "bpmn cancel parse should default local workflow-state store",
        ),
        "bpmn cancel command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Cancel(BpmnCancelCliCommand {
            instance_id: "wf_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
        })
    );
}
