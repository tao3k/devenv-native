use super::*;

#[cfg(feature = "duckdb")]
use crate::test_exports::BpmnResumeCliCommand;

#[cfg(not(feature = "duckdb"))]
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

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_defaults_resume_to_local_duckdb_checkpoint_backend() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "resume",
                "--bpmn",
                "fixtures/wait.bpmn",
                "--instance-id",
                "wf_wait",
            ])),
            "bpmn resume parse should default local workflow-state store",
        ),
        "bpmn resume command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Resume(BpmnResumeCliCommand {
            bpmn_path: PathBuf::from("fixtures/wait.bpmn"),
            dmn_paths: Vec::new(),
            instance_id: "wf_wait".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
            continue_until_human_boundary: false,
        })
    );
}
