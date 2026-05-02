#[cfg(feature = "duckdb")]
use super::{
    BpmnCliCheckpointBackend, BpmnCliCommand, BpmnTaskCompleteCliCommand, BpmnTaskCompleteCliKind,
    PathBuf, must_ok, must_some, parse_bpmn_command, to_args,
};
#[cfg(not(feature = "duckdb"))]
use super::{parse_bpmn_command, to_args};

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
        "--token-id",
        "0",
        "--process-id",
        "review",
        "--activity-id",
        "review_task",
        "--kind",
        "user",
        "--data-json",
        r#"{"approved":true}"#,
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
fn parse_bpmn_command_accepts_tasks_complete_typed_payload() {
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
                "--token-id",
                "7",
                "--process-id",
                "review",
                "--activity-id",
                "review_task",
                "--kind",
                "user",
                "--data-json",
                r#"{"approved":true}"#,
                "--claimant",
                "alice",
            ])),
            "bpmn tasks complete parse should accept explicit typed payload",
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
            token_id: 7,
            process_id: "review".to_string(),
            activity_id: "review_task".to_string(),
            kind: BpmnTaskCompleteCliKind::User,
            data_json: r#"{"approved":true}"#.to_string(),
            claimant: Some("alice".to_string()),
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            continue_until_human_boundary: false,
        })
    );
}

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_rejects_tasks_complete_without_identity_payload() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "tasks",
        "complete",
        "--bpmn",
        "fixtures/review.bpmn",
        "--instance-id",
        "wf_service",
        "--token-id",
        "7",
    ])) {
        Ok(command) => panic!("missing identity payload should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--process-id <id>` for `bpmn tasks complete` command")
    );
}

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_accepts_tasks_complete_continuation_fixture() {
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
                "--token-id",
                "7",
                "--process-id",
                "review",
                "--activity-id",
                "review_task",
                "--kind",
                "user",
                "--data-json",
                r#"{"approved":true}"#,
                "--host-fixture",
                "fixtures/host.json",
                "--continue-until-human-boundary",
            ])),
            "bpmn tasks complete parse should accept continuation fixtures",
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
            token_id: 7,
            process_id: "review".to_string(),
            activity_id: "review_task".to_string(),
            kind: BpmnTaskCompleteCliKind::User,
            data_json: r#"{"approved":true}"#.to_string(),
            claimant: None,
            host_fixture_path: Some(PathBuf::from("fixtures/host.json")),
            event_fixture_path: None,
            trace_stream: false,
            continue_until_human_boundary: true,
        })
    );
}
