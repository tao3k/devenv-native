use super::*;

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
    let pending =
        seed_checkpointed_cli_pending_task(bpmn_path, &duckdb_path, &runtime_env, instance_id)
            .await;

    let claim_command = BpmnTaskClaimCliCommand {
        instance_id: instance_id.to_string(),
        checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
        token_id: pending.token,
        process_id: pending.process.clone(),
        activity_id: pending.activity.clone(),
        claimant: "alice".to_string(),
    };
    let claim_output = must_ok(
        run_bpmn_task_claim_command_with_runtime_env(&claim_command, Some(&runtime_env), None)
            .await,
        "bpmn tasks claim should use checkpointed control service",
    );
    assert_claim_rendered_summary(&claim_output.rendered);

    let alice_worklist = must_ok(
        run_bpmn_task_worklist_command_with_runtime_env(
            &BpmnTaskWorklistCliCommand {
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                claimant: Some("alice".to_string()),
                assignment_resource: None,
                lane: None,
            },
            Some(&runtime_env),
        )
        .await,
        "bpmn tasks worklist should include matching claimed work",
    );
    assert_worklist_rendered(&alice_worklist.rendered, "claim=alice");

    let release_output = must_ok(
        run_bpmn_task_release_command_with_runtime_env(
            &BpmnTaskReleaseCliCommand {
                instance_id: instance_id.to_string(),
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                token_id: pending.token,
                process_id: pending.process,
                activity_id: pending.activity,
                claimant: "alice".to_string(),
            },
            Some(&runtime_env),
            None,
        )
        .await,
        "bpmn tasks release should use checkpointed control service",
    );
    assert_release_rendered_summary(&release_output.rendered);

    let status_output = must_ok(
        run_bpmn_status_command_with_runtime_env(
            &BpmnStatusCliCommand {
                instance_id: instance_id.to_string(),
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                bpmn_path: None,
                dmn_paths: Vec::new(),
            },
            Some(&runtime_env),
        )
        .await,
        "bpmn status should render human-task lifecycle summary",
    );
    assert_lifecycle_summary(&status_output.rendered, "3", "released");

    let unclaimed_worklist = must_ok(
        run_bpmn_task_worklist_command_with_runtime_env(
            &BpmnTaskWorklistCliCommand {
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                claimant: None,
                assignment_resource: None,
                lane: None,
            },
            Some(&runtime_env),
        )
        .await,
        "bpmn tasks worklist should include released unclaimed work",
    );
    assert_worklist_rendered(&unclaimed_worklist.rendered, "claim=unclaimed");
}

#[cfg(feature = "duckdb")]
fn assert_claim_rendered_summary(rendered: &str) {
    assert!(rendered.contains("# BPMN Task Claim"));
    assert!(rendered.contains("Claimant: alice"));
    assert!(rendered.contains("Claim status: claimed"));
    assert_lifecycle_summary(rendered, "2", "claimed");
    assert!(rendered.contains("claimant=alice"));
    assert!(rendered.contains("Authorization: not evaluated"));
}

#[cfg(feature = "duckdb")]
fn assert_release_rendered_summary(rendered: &str) {
    assert!(rendered.contains("# BPMN Task Release"));
    assert!(rendered.contains("Claim status: unclaimed"));
    assert_lifecycle_summary(rendered, "3", "released");
}

#[cfg(feature = "duckdb")]
fn assert_lifecycle_summary(rendered: &str, count: &str, last_event: &str) {
    assert!(rendered.contains(&format!("Human task lifecycle events: {count}")));
    assert!(rendered.contains(&format!("Last human task event: {last_event}")));
}

#[cfg(feature = "duckdb")]
fn assert_worklist_rendered(rendered: &str, claim_summary: &str) {
    assert!(rendered.contains("Item count: 1"));
    assert!(rendered.contains(claim_summary));
    assert!(rendered.contains("activity=review_task"));
}

#[cfg(feature = "duckdb")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_task_complete_renders_human_task_lifecycle_summary() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-cli-complete-lifecycle.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let instance_id = "wf_task_cli_complete_lifecycle";
    let pending = seed_checkpointed_cli_pending_task(
        bpmn_path.clone(),
        &duckdb_path,
        &runtime_env,
        instance_id,
    )
    .await;

    let claim_output = must_ok(
        run_bpmn_task_claim_command_with_runtime_env(
            &BpmnTaskClaimCliCommand {
                instance_id: instance_id.to_string(),
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                token_id: pending.token,
                process_id: pending.process.clone(),
                activity_id: pending.activity.clone(),
                claimant: "alice".to_string(),
            },
            Some(&runtime_env),
            None,
        )
        .await,
        "bpmn tasks claim should persist claimant metadata before completion",
    );
    assert!(
        claim_output
            .rendered
            .contains("Last human task event: claimed")
    );

    let complete_output = must_ok(
        run_bpmn_task_complete_command_with_runtime_env(
            &BpmnTaskCompleteCliCommand {
                bpmn_path,
                dmn_paths: Vec::new(),
                instance_id: instance_id.to_string(),
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                token_id: pending.token,
                process_id: pending.process,
                activity_id: pending.activity,
                kind: BpmnTaskCompleteCliKind::User,
                data_json: r#"{"answer":"approved"}"#.to_string(),
                claimant: Some("alice".to_string()),
                host_fixture_path: None,
                event_fixture_path: None,
                trace_stream: false,
                continue_until_human_boundary: false,
            },
            Some(&runtime_env),
            None,
        )
        .await,
        "bpmn tasks complete should render checkpointed lifecycle summary",
    );

    assert_rendered_contains(&complete_output.rendered, "# BPMN Task Complete");
    assert_rendered_contains(&complete_output.rendered, "Human task lifecycle events: 3");
    assert_rendered_contains(
        &complete_output.rendered,
        "Last human task event: completed",
    );
    assert_rendered_contains(&complete_output.rendered, "claimant=alice");
}

fn assert_rendered_contains(rendered: &str, expected: &str) {
    assert!(
        rendered.contains(expected),
        "expected rendered output to contain {expected:?}:\n{rendered}"
    );
}

#[cfg(feature = "duckdb")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_task_worklist_renders_human_task_abi_fields() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_interactive_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-cli-worklist-abi.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let instance_id = "wf_task_cli_worklist_abi";
    let pending =
        seed_checkpointed_cli_pending_task(bpmn_path, &duckdb_path, &runtime_env, instance_id)
            .await;

    let claim_output = must_ok(
        run_bpmn_task_claim_command_with_runtime_env(
            &BpmnTaskClaimCliCommand {
                instance_id: instance_id.to_string(),
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                token_id: pending.token,
                process_id: pending.process.clone(),
                activity_id: pending.activity.clone(),
                claimant: "alice".to_string(),
            },
            Some(&runtime_env),
            None,
        )
        .await,
        "bpmn tasks claim should persist claimant metadata for worklist parity",
    );
    assert!(claim_output.rendered.contains("Claim status: claimed"));

    let worklist = must_ok(
        run_bpmn_task_worklist_command_with_runtime_env(
            &BpmnTaskWorklistCliCommand {
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                claimant: Some("alice".to_string()),
                assignment_resource: None,
                lane: None,
            },
            Some(&runtime_env),
        )
        .await,
        "bpmn tasks worklist should render the human-task ABI field set",
    );

    assert!(worklist.rendered.contains("Item count: 1"));
    assert!(worklist.rendered.contains(
        "Authorization: not evaluated; BPMN assignment and lane metadata are routing-only."
    ));
    assert!(worklist.rendered.contains(&format!(
        "- {instance_id} | token#{} | process=review | activity=review_task | kind=user",
        pending.token
    )));
    assert!(worklist.rendered.contains("claim=alice"));
    assert!(
        worklist
            .rendered
            .contains("form=choice_input result=answer fields=feedback?")
    );
    assert!(
        worklist
            .rendered
            .contains("assignment=human_performer:reviewer:expr=users.alice;potential_owner:review_team:ref=reviewers")
    );
    assert!(
        worklist
            .rendered
            .contains("lane=Reviewer Lane id=Lane_Reviewer set=Ownership")
    );
}

#[cfg(feature = "duckdb")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_task_worklist_filters_assignment_routing_metadata() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_interactive_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-cli-worklist-routing.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let instance_id = "wf_task_cli_worklist_routing";
    let pending =
        seed_checkpointed_cli_pending_task(bpmn_path, &duckdb_path, &runtime_env, instance_id)
            .await;

    let reviewers_worklist = rendered_task_worklist_for_filters(
        &runtime_env,
        None,
        Some("reviewers"),
        None,
        "bpmn tasks worklist should filter by assignment resource",
    )
    .await;
    assert!(reviewers_worklist.contains("Assignment resource filter: reviewers"));
    assert!(reviewers_worklist.contains("Item count: 1"));
    assert!(reviewers_worklist.contains(&format!(
        "- {instance_id} | token#{} | process=review | activity=review_task | kind=user",
        pending.token
    )));

    let finance_worklist = rendered_task_worklist_for_filters(
        &runtime_env,
        None,
        Some("finance"),
        None,
        "bpmn tasks worklist should return no non-matching assignment resource",
    )
    .await;
    assert!(finance_worklist.contains("Item count: 0"));

    let claim_output = must_ok(
        run_bpmn_task_claim_command_with_runtime_env(
            &BpmnTaskClaimCliCommand {
                instance_id: instance_id.to_string(),
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                token_id: pending.token,
                process_id: pending.process,
                activity_id: pending.activity,
                claimant: "alice".to_string(),
            },
            Some(&runtime_env),
            None,
        )
        .await,
        "bpmn tasks claim should persist claimant metadata",
    );
    assert!(claim_output.rendered.contains("Claim status: claimed"));

    let composed_worklist = rendered_task_worklist_for_filters(
        &runtime_env,
        Some("alice"),
        Some("review_team"),
        None,
        "claimant and assignment routing filters should compose in CLI worklist",
    )
    .await;
    assert!(composed_worklist.contains("Item count: 1"));
    assert!(composed_worklist.contains("claim=alice"));

    let lane_worklist = rendered_task_worklist_for_filters(
        &runtime_env,
        Some("alice"),
        Some("reviewers"),
        Some("Reviewer Lane"),
        "claimant, assignment, and lane filters should compose in CLI worklist",
    )
    .await;
    assert!(lane_worklist.contains("Lane filter: Reviewer Lane"));
    assert!(lane_worklist.contains("Item count: 1"));
    assert!(lane_worklist.contains("claim=alice"));

    let hidden_by_lane = rendered_task_worklist_for_filters(
        &runtime_env,
        Some("alice"),
        Some("reviewers"),
        Some("Finance Lane"),
        "non-matching lane filter should return no CLI worklist items",
    )
    .await;
    assert!(hidden_by_lane.contains("Lane filter: Finance Lane"));
    assert!(hidden_by_lane.contains("Item count: 0"));
}

#[cfg(feature = "duckdb")]
async fn rendered_task_worklist_for_filters(
    runtime_env: &QianjiRuntimeEnv,
    claimant: Option<&str>,
    assignment_resource: Option<&str>,
    lane: Option<&str>,
    message: &str,
) -> String {
    must_ok(
        run_bpmn_task_worklist_command_with_runtime_env(
            &BpmnTaskWorklistCliCommand {
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                claimant: claimant.map(ToString::to_string),
                assignment_resource: assignment_resource.map(ToString::to_string),
                lane: lane.map(ToString::to_string),
            },
            Some(runtime_env),
        )
        .await,
        message,
    )
    .rendered
}

#[cfg(feature = "duckdb")]
struct CheckpointedCliPendingTask {
    token: u64,
    process: String,
    activity: String,
}

#[cfg(feature = "duckdb")]
async fn seed_checkpointed_cli_pending_task(
    bpmn_path: std::path::PathBuf,
    duckdb_path: &std::path::Path,
    runtime_env: &QianjiRuntimeEnv,
    instance_id: &str,
) -> CheckpointedCliPendingTask {
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
            Some(runtime_env),
            None,
        )
        .await,
        "bpmn run should persist a checkpointed pending user task",
    );
    assert!(start_output.rendered.contains("Pending host work: 1"));

    let checkpoint = must_some(
        must_ok(
            xiuxian_qianji::QianjiBpmnCheckpointStore::duckdb(duckdb_path.to_path_buf())
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
    CheckpointedCliPendingTask {
        token: pending.token_id,
        process: pending
            .process_id
            .clone()
            .unwrap_or_else(|| checkpoint.state.process.process_id.as_ref().to_string()),
        activity: pending
            .activity_id
            .clone()
            .unwrap_or_else(|| format!("node#{}", pending.node_index)),
    }
}
