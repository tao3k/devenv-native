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
async fn host_session_surfaces_service_task_without_host_fixture() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_service_task_bundle(&temp_dir);

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::HostSession(BpmnHostSessionCliCommand {
            start: BpmnRunCliCommand {
                bpmn_path,
                dmn_paths: Vec::new(),
                process_id: "review".to_string(),
                instance_id: "wf_host_session_service_external".to_string(),
                context_json: Some("{\"risk\":\"high\"}".to_string()),
                start_at_node_id: None,
                checkpoint_backend: None,
                host_fixture_path: None,
                event_fixture_path: None,
                trace_stream: false,
                external_host: true,
                continue_until_human_boundary: true,
            },
        }))
        .await,
        "host-session should expose service work instead of dispatching without a fixture",
    );

    assert_eq!(output.exit_code, 0);
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

#[cfg(feature = "duckdb")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_task_claim_worklist_release_commands_use_checkpointed_control_service() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-cli-claim.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let instance_id = "wf_task_cli_claim";

    let start_output = must_ok(
        run_bpmn_run_command_with_runtime_env(
            &BpmnRunCliCommand {
                bpmn_path,
                dmn_paths: Vec::new(),
                process_id: "review".to_string(),
                instance_id: instance_id.to_string(),
                context_json: Some("{}".to_string()),
                start_at_node_id: None,
                checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
                host_fixture_path: None,
                event_fixture_path: None,
                trace_stream: false,
                external_host: true,
                continue_until_human_boundary: false,
            },
            Some(&runtime_env),
            None,
        )
        .await,
        "bpmn run should persist a checkpointed pending user task",
    );
    assert!(start_output.rendered.contains("Pending host work: 1"));

    let checkpoint = must_some(
        must_ok(
            xiuxian_qianji::QianjiBpmnCheckpointStore::duckdb(duckdb_path.clone())
                .load(instance_id)
                .await,
            "checkpoint should load after external-host user task boundary",
        ),
        "checkpoint should exist after external-host user task boundary",
    );
    let pending = checkpoint
        .state
        .pending_host_work
        .first()
        .unwrap_or_else(|| panic!("checkpoint should contain pending human work"));
    let process_id = pending
        .process_id
        .clone()
        .unwrap_or_else(|| checkpoint.state.process.process_id.as_ref().to_string());
    let activity_id = pending
        .activity_id
        .clone()
        .unwrap_or_else(|| format!("node#{}", pending.node_index));
    let token_id = pending.token_id;

    let claim_command = BpmnTaskClaimCliCommand {
        instance_id: instance_id.to_string(),
        checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
        token_id,
        process_id: process_id.clone(),
        activity_id: activity_id.clone(),
        claimant: "alice".to_string(),
    };
    let claim_output = must_ok(
        run_bpmn_task_claim_command_with_runtime_env(&claim_command, Some(&runtime_env), None)
            .await,
        "bpmn tasks claim should use checkpointed control service",
    );
    assert!(claim_output.rendered.contains("# BPMN Task Claim"));
    assert!(claim_output.rendered.contains("Claimant: alice"));
    assert!(claim_output.rendered.contains("Claim status: claimed"));
    assert!(
        claim_output
            .rendered
            .contains("Authorization: not evaluated")
    );

    let alice_worklist = must_ok(
        run_bpmn_task_worklist_command_with_runtime_env(
            &BpmnTaskWorklistCliCommand {
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                claimant: Some("alice".to_string()),
            },
            Some(&runtime_env),
        )
        .await,
        "bpmn tasks worklist should include matching claimed work",
    );
    assert!(alice_worklist.rendered.contains("Item count: 1"));
    assert!(alice_worklist.rendered.contains("claim=alice"));
    assert!(alice_worklist.rendered.contains("activity=review_task"));

    let release_output = must_ok(
        run_bpmn_task_release_command_with_runtime_env(
            &BpmnTaskReleaseCliCommand {
                instance_id: instance_id.to_string(),
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                token_id,
                process_id,
                activity_id,
                claimant: "alice".to_string(),
            },
            Some(&runtime_env),
            None,
        )
        .await,
        "bpmn tasks release should use checkpointed control service",
    );
    assert!(release_output.rendered.contains("# BPMN Task Release"));
    assert!(release_output.rendered.contains("Claim status: unclaimed"));

    let unclaimed_worklist = must_ok(
        run_bpmn_task_worklist_command_with_runtime_env(
            &BpmnTaskWorklistCliCommand {
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                claimant: None,
            },
            Some(&runtime_env),
        )
        .await,
        "bpmn tasks worklist should include released unclaimed work",
    );
    assert!(unclaimed_worklist.rendered.contains("Item count: 1"));
    assert!(unclaimed_worklist.rendered.contains("claim=unclaimed"));
}
