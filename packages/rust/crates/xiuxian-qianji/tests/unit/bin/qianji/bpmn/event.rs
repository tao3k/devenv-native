use super::*;

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_completes_waiting_bundle_with_event_fixture() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_event_wait_bundle(&temp_dir);
    let fixture_path = write_json_fixture(
        temp_dir.path().join("event-fixture.json"),
        &json!({
            "event_polls": {
                "wait_message": {
                    "ready": true,
                    "data": {
                        "approved": true,
                        "source": "event_fixture"
                    }
                }
            }
        }),
    );

    let output = must_ok(
        boxed_future(run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "wait_flow".to_string(),
            instance_id: "wf_wait".to_string(),
            context_json: Some("{\"amount\":7}".to_string()),
            start_at_node_id: None,
            checkpoint_backend: None,
            host_fixture_path: None,
            event_fixture_path: Some(fixture_path.clone()),
            trace_stream: false,
            external_host: false,
            continue_until_human_boundary: false,
        })))
        .await,
        "bpmn run should resolve waiting bundle with event fixture",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Outcome: completed"));
    assert!(
        output
            .rendered
            .contains(&format!("Event fixture: {}", fixture_path.display()))
    );
    assert!(output.rendered.contains("\"approved\": true"));
    assert!(output.rendered.contains("\"source\": \"event_fixture\""));
}

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_waiting_event_race_renders_competing_wait_diagnostics() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_event_race_bundle(&temp_dir);

    let output = must_ok(
        boxed_future(run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "event_race".to_string(),
            instance_id: "wf_event_race".to_string(),
            context_json: Some("{\"amount\":7}".to_string()),
            start_at_node_id: None,
            checkpoint_backend: None,
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
            continue_until_human_boundary: false,
        })))
        .await,
        "bpmn run should keep competing waits visible when no event fixture is supplied",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Outcome: waiting_external_event"));
    assert!(output.rendered.contains("Wait registrations: 2"));
    assert!(output.rendered.contains("## Wait Registrations"));
    assert!(output.rendered.contains("Competition gateway: wait_race"));
    assert!(
        output
            .rendered
            .contains("Event fixture key: wait_message|wait_timer")
    );
    assert!(output.rendered.contains(
        "- wait_message | kind=external_event | event=message | ref=invoice_received | name=InvoiceReceived | correlation=invoice_received"
    ));
    assert!(
        output
            .rendered
            .contains("- wait_timer | kind=timer | event=timer")
    );
    assert!(output.rendered.contains("timer=duration:PT5M"));
}

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_completes_event_race_bundle_with_competition_event_fixture() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_event_race_bundle(&temp_dir);
    let fixture_path = write_json_fixture(
        temp_dir.path().join("event-race-fixture.json"),
        &json!({
            "event_polls": {
                "wait_message|wait_timer": {
                    "ready": true,
                    "winning_wait_id": "wait_message",
                    "data": {
                        "approved": true,
                        "winner": "message_fixture"
                    }
                }
            }
        }),
    );

    let output = must_ok(
        boxed_future(run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "event_race".to_string(),
            instance_id: "wf_event_race".to_string(),
            context_json: Some("{\"amount\":7}".to_string()),
            start_at_node_id: None,
            checkpoint_backend: None,
            host_fixture_path: None,
            event_fixture_path: Some(fixture_path.clone()),
            trace_stream: false,
            external_host: false,
            continue_until_human_boundary: false,
        })))
        .await,
        "bpmn run should resolve competing waits with an explicit event fixture winner",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Outcome: completed"));
    assert!(
        output
            .rendered
            .contains(&format!("Event fixture: {}", fixture_path.display()))
    );
    assert!(output.rendered.contains("\"approved\": true"));
    assert!(output.rendered.contains("\"winner\": \"message_fixture\""));
}
