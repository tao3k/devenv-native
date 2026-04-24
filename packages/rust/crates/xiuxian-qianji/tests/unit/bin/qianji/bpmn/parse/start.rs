use super::*;

#[test]
fn parse_bpmn_command_accepts_fresh_start_with_dmn_sources() {
    let command = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "start",
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
            "bpmn start parse should succeed",
        ),
        "bpmn start command should be detected",
    );

    assert_eq!(
        command,
        BpmnCliCommand::Start(BpmnStartCliCommand {
            bpmn_path: PathBuf::from("fixtures/review.bpmn"),
            dmn_paths: vec![PathBuf::from("fixtures/review.dmn")],
            process_id: "review".to_string(),
            instance_id: "wf_review".to_string(),
            context_json: Some("{\"risk\":\"high\"}".to_string()),
            checkpoint_backend: None,
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
        })
    );
}

#[test]
fn parse_bpmn_command_rejects_fresh_start_without_context() {
    let error = match parse_bpmn_command(&to_args(&[
        "qianji",
        "bpmn",
        "start",
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
            .contains("missing `--context-json <json>` for fresh `bpmn start` command")
    );
}
