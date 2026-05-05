#[cfg(feature = "duckdb")]
use super::{
    BpmnCliCheckpointBackend, BpmnCliCommand, BpmnInterruptCliCommand, must_ok, must_some,
    parse_bpmn_command, to_args,
};
#[cfg(not(feature = "duckdb"))]
use super::{parse_bpmn_command, to_args};

#[cfg(not(feature = "duckdb"))]
#[test]
fn parse_bpmn_command_rejects_interrupt_without_checkpoint_backend() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "interrupt",
        "--instance-id",
        "wf_wait",
    ])) {
        Ok(command) => panic!("missing interrupt checkpoint backend should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing checkpoint backend for `bpmn interrupt`")
    );
}

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_defaults_interrupt_to_local_duckdb_checkpoint_backend() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "interrupt",
                "--instance-id",
                "wf_wait",
            ])),
            "bpmn interrupt parse should default local workflow-state store",
        ),
        "bpmn interrupt command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Interrupt(BpmnInterruptCliCommand {
            instance_id: "wf_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
        })
    );
}

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_accepts_stop_alias_for_interrupt() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "stop",
                "--instance-id",
                "wf_wait",
            ])),
            "bpmn stop parse should use interrupt semantics",
        ),
        "bpmn stop command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Interrupt(BpmnInterruptCliCommand {
            instance_id: "wf_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
        })
    );
}
