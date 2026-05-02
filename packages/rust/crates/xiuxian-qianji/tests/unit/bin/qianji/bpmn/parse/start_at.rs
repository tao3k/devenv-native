use super::{
    BpmnCliCheckpointBackend, BpmnCliCommand, BpmnStartAtCliCommand, PathBuf, must_ok, must_some,
    parse_bpmn_command, to_args,
};

#[test]
fn parse_bpmn_command_accepts_start_at_node() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "start-at",
                "--bpmn",
                "fixtures/review.bpmn",
                "--process",
                "review",
                "--node",
                "Task_Ask",
                "--instance-id",
                "wf_review_question",
                "--context-json",
                "{\"currentQuestion\":\"Ready?\"}",
                "--external-host",
            ])),
            "bpmn start-at parse should succeed",
        ),
        "bpmn command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::StartAt(BpmnStartAtCliCommand {
            bpmn_path: PathBuf::from("fixtures/review.bpmn"),
            dmn_paths: Vec::new(),
            process_id: "review".to_string(),
            instance_id: "wf_review_question".to_string(),
            context_json: Some("{\"currentQuestion\":\"Ready?\"}".to_string()),
            start_at_node_id: Some("Task_Ask".to_string()),
            #[cfg(feature = "duckdb")]
            checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
            #[cfg(not(feature = "duckdb"))]
            checkpoint_backend: None,
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: true,
            continue_until_human_boundary: false,
        })
    );
}

#[test]
fn parse_bpmn_command_rejects_start_at_without_node() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "start-at",
        "--bpmn",
        "fixtures/review.bpmn",
        "--process",
        "review",
        "--instance-id",
        "wf_review_question",
        "--context-json",
        "{}",
    ])) {
        Ok(command) => panic!("missing node should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--node <id>` for `bpmn start-at` command")
    );
}

#[test]
fn parse_bpmn_command_rejects_run_node_alias() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "run",
        "--bpmn",
        "fixtures/review.bpmn",
        "--process",
        "review",
        "--node",
        "Task_Ask",
        "--instance-id",
        "wf_review_question",
        "--context-json",
        "{}",
    ])) {
        Ok(command) => panic!("run --node should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("unsupported `bpmn run` option `--node`; use `bpmn start-at`")
    );
}
