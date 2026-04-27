use super::*;

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_completes_linear_bundle() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_linear_bundle(&temp_dir);

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "linear".to_string(),
            instance_id: "wf_review".to_string(),
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
        "bpmn run should complete linear bundle",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# BPMN Run"));
    assert!(output.rendered.contains("Outcome: completed"));
    assert!(output.rendered.contains("Checkpoint backend: none"));
    assert!(output.rendered.contains("Host fixture: none"));
    assert!(output.rendered.contains("Event fixture: none"));
    assert!(output.rendered.contains("\"risk\": \"high\""));
}

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_completes_service_task_bundle_with_host_fixture() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_service_task_bundle(&temp_dir);
    let fixture_path = write_json_fixture(
        temp_dir.path().join("host-fixture.json"),
        &json!({
            "service_tasks": {
                "review_task": {
                    "data": {
                        "approved": true,
                        "reviewed_by": "fixture"
                    }
                }
            }
        }),
    );

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "review".to_string(),
            instance_id: "wf_service".to_string(),
            context_json: Some("{\"risk\":\"high\"}".to_string()),
            start_at_node_id: None,
            checkpoint_backend: None,
            host_fixture_path: Some(fixture_path.clone()),
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
            continue_until_human_boundary: false,
        }))
        .await,
        "bpmn run should complete service task bundle with host fixture",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Outcome: completed"));
    assert!(
        output
            .rendered
            .contains(&format!("Host fixture: {}", fixture_path.display()))
    );
    assert!(output.rendered.contains("\"approved\": true"));
    assert!(output.rendered.contains("\"reviewed_by\": \"fixture\""));
}

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_completes_send_task_bundle_with_host_fixture() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_send_task_bundle(&temp_dir);
    let fixture_path = write_json_fixture(
        temp_dir.path().join("send-host-fixture.json"),
        &json!({
            "send_tasks": {
                "send_invoice_message": {
                    "data": {
                        "sent": true,
                        "transport": "fixture"
                    }
                }
            }
        }),
    );

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "send_flow".to_string(),
            instance_id: "wf_send".to_string(),
            context_json: Some("{\"risk\":\"high\"}".to_string()),
            start_at_node_id: None,
            checkpoint_backend: None,
            host_fixture_path: Some(fixture_path.clone()),
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
            continue_until_human_boundary: false,
        }))
        .await,
        "bpmn run should complete send task bundle with host fixture",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Outcome: completed"));
    assert!(
        output
            .rendered
            .contains(&format!("Host fixture: {}", fixture_path.display()))
    );
    assert!(output.rendered.contains("\"sent\": true"));
    assert!(output.rendered.contains("\"transport\": \"fixture\""));
}

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_completes_business_rule_bundle_with_host_fixture() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bundle = write_business_rule_bundle(&temp_dir);
    let fixture_path = write_json_fixture(
        temp_dir.path().join("business-rule-fixture.json"),
        &json!({
            "business_rule_tasks": {
                "review_task": {
                    "output": {
                        "approval": "manual_review",
                        "reason": "fixture_override"
                    },
                    "matched_rule_ids": ["fixture_rule_review"]
                }
            }
        }),
    );

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path: bundle.bpmn_path,
            dmn_paths: vec![bundle.dmn_path],
            process_id: "review".to_string(),
            instance_id: "wf_business_rule".to_string(),
            context_json: Some("{\"risk\":\"high\"}".to_string()),
            start_at_node_id: None,
            checkpoint_backend: None,
            host_fixture_path: Some(fixture_path.clone()),
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
            continue_until_human_boundary: false,
        }))
        .await,
        "bpmn run should complete business-rule bundle with host fixture",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Outcome: completed"));
    assert!(
        output
            .rendered
            .contains(&format!("Host fixture: {}", fixture_path.display()))
    );
    assert!(output.rendered.contains("\"approval\": \"manual_review\""));
    assert!(output.rendered.contains("\"reason\": \"fixture_override\""));
}
