use super::*;

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_start_command_completes_linear_bundle() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_linear_bundle(&temp_dir);

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::Start(BpmnStartCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "linear".to_string(),
            instance_id: "wf_start".to_string(),
            context_json: Some("{\"risk\":\"high\"}".to_string()),
            start_at_node_id: None,
            checkpoint_backend: None,
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
            continue_until_human_boundary: false,
        }))
        .await,
        "bpmn start should complete linear bundle",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# BPMN Start"));
    assert!(output.rendered.contains("Outcome: completed"));
    assert!(output.rendered.contains("Checkpoint backend: none"));
    assert!(output.rendered.contains("Host fixture: none"));
    assert!(output.rendered.contains("Event fixture: none"));
    assert!(output.rendered.contains("\"risk\": \"high\""));
}
