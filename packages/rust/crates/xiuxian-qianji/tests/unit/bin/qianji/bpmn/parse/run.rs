use super::*;

#[test]
fn parse_bpmn_command_accepts_fresh_run_with_dmn_sources() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "run",
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
            ])),
            "bpmn parse should succeed",
        ),
        "bpmn command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path: PathBuf::from("fixtures/review.bpmn"),
            dmn_paths: vec![PathBuf::from("fixtures/review.dmn")],
            process_id: "review".to_string(),
            instance_id: "wf_review".to_string(),
            context_json: Some("{\"risk\":\"high\"}".to_string()),
            #[cfg(feature = "duckdb")]
            checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
            #[cfg(not(feature = "duckdb"))]
            checkpoint_backend: None,
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
        })
    );
}

#[test]
fn parse_bpmn_command_accepts_host_fixture() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "run",
                "--bpmn",
                "fixtures/review.bpmn",
                "--process",
                "review",
                "--instance-id",
                "wf_review",
                "--context-json",
                "{\"risk\":\"high\"}",
                "--host-fixture",
                "fixtures/host.json",
            ])),
            "bpmn parse with host fixture should succeed",
        ),
        "bpmn command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path: PathBuf::from("fixtures/review.bpmn"),
            dmn_paths: Vec::new(),
            process_id: "review".to_string(),
            instance_id: "wf_review".to_string(),
            context_json: Some("{\"risk\":\"high\"}".to_string()),
            #[cfg(feature = "duckdb")]
            checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
            #[cfg(not(feature = "duckdb"))]
            checkpoint_backend: None,
            host_fixture_path: Some(PathBuf::from("fixtures/host.json")),
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
        })
    );
}

#[test]
fn parse_bpmn_command_accepts_event_fixture() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "run",
                "--bpmn",
                "fixtures/wait.bpmn",
                "--process",
                "wait_flow",
                "--instance-id",
                "wf_wait",
                "--context-json",
                "{}",
                "--event-fixture",
                "fixtures/events.json",
            ])),
            "bpmn parse with event fixture should succeed",
        ),
        "bpmn command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path: PathBuf::from("fixtures/wait.bpmn"),
            dmn_paths: Vec::new(),
            process_id: "wait_flow".to_string(),
            instance_id: "wf_wait".to_string(),
            context_json: Some("{}".to_string()),
            #[cfg(feature = "duckdb")]
            checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
            #[cfg(not(feature = "duckdb"))]
            checkpoint_backend: None,
            host_fixture_path: None,
            event_fixture_path: Some(PathBuf::from("fixtures/events.json")),
            trace_stream: false,
            external_host: false,
        })
    );
}

#[test]
fn parse_bpmn_command_accepts_trace_stream() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "run",
                "--bpmn",
                "fixtures/review.bpmn",
                "--process",
                "review",
                "--instance-id",
                "wf_review",
                "--context-json",
                "{}",
                "--trace-stream",
            ])),
            "bpmn parse with trace stream should succeed",
        ),
        "bpmn command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path: PathBuf::from("fixtures/review.bpmn"),
            dmn_paths: Vec::new(),
            process_id: "review".to_string(),
            instance_id: "wf_review".to_string(),
            context_json: Some("{}".to_string()),
            #[cfg(feature = "duckdb")]
            checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
            #[cfg(not(feature = "duckdb"))]
            checkpoint_backend: None,
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: true,
            external_host: false,
        })
    );
}

#[cfg(not(feature = "duckdb"))]
#[test]
fn parse_bpmn_command_rejects_fresh_run_without_context() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "run",
        "--bpmn",
        "fixtures/review.bpmn",
        "--process",
        "review",
        "--instance-id",
        "wf_review",
    ])) {
        Ok(command) => panic!("missing context should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--context-json <json>` for fresh `bpmn run` command")
    );
}

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_defaults_fresh_run_without_context_to_local_duckdb() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "run",
                "--bpmn",
                "fixtures/review.bpmn",
                "--process",
                "review",
                "--instance-id",
                "wf_review",
            ])),
            "bpmn parse should default local workflow-state store",
        ),
        "bpmn command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path: PathBuf::from("fixtures/review.bpmn"),
            dmn_paths: Vec::new(),
            process_id: "review".to_string(),
            instance_id: "wf_review".to_string(),
            context_json: None,
            checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
        })
    );
}
