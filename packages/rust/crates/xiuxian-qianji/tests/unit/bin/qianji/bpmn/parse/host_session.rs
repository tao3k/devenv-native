use super::{
    BpmnCliCheckpointBackend, BpmnCliCommand, BpmnHostSessionCliCommand, BpmnRunCliCommand,
    PathBuf, must_ok, must_some, parse_bpmn_command, to_args,
};

#[test]
fn parse_bpmn_command_accepts_host_session() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "host-session",
                "--bpmn",
                "fixtures/review.bpmn",
                "--dmn",
                "fixtures/review.dmn",
                "--process",
                "review",
                "--instance-id",
                "wf_review",
                "--context-json",
                "{\"risk\":\"high\"}",
                "--host-fixture",
                "fixtures/host.json",
                "--trace-stream",
            ])),
            "bpmn host-session parse should succeed",
        ),
        "bpmn command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::HostSession(BpmnHostSessionCliCommand {
            start: BpmnRunCliCommand {
                bpmn_path: PathBuf::from("fixtures/review.bpmn"),
                dmn_paths: vec![PathBuf::from("fixtures/review.dmn")],
                process_id: "review".to_string(),
                instance_id: "wf_review".to_string(),
                context_json: Some("{\"risk\":\"high\"}".to_string()),
                start_at_node_id: None,
                #[cfg(feature = "duckdb")]
                checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
                #[cfg(not(feature = "duckdb"))]
                checkpoint_backend: None,
                host_fixture_path: Some(PathBuf::from("fixtures/host.json")),
                event_fixture_path: None,
                trace_stream: true,
                external_host: true,
                continue_until_human_boundary: true,
            },
        })
    );
}

#[test]
fn parse_bpmn_command_accepts_host_session_start_at_node() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "host-session",
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
            ])),
            "bpmn host-session start-at parse should succeed",
        ),
        "bpmn command should be detected",
    );

    let BpmnCliCommand::HostSession(command) = command else {
        panic!("expected host-session command");
    };
    assert_eq!(command.start.start_at_node_id, Some("Task_Ask".to_string()));
    assert!(command.start.external_host);
    assert!(command.start.continue_until_human_boundary);
}
