use super::*;

#[cfg(feature = "duckdb")]
use crate::test_exports::BpmnTaskCompleteCliCommand;

#[cfg(not(feature = "duckdb"))]
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

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_defaults_tasks_complete_to_local_duckdb_checkpoint_backend() {
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
            ])),
            "bpmn tasks complete parse should default local workflow-state store",
        ),
        "bpmn tasks complete command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::TaskComplete(BpmnTaskCompleteCliCommand {
            bpmn_path: PathBuf::from("fixtures/review.bpmn"),
            dmn_paths: Vec::new(),
            instance_id: "wf_service".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
        })
    );
}
