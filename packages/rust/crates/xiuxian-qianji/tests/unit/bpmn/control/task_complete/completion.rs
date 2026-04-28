use super::*;

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_accepts_typed_user_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-action.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path).await;

    let complete_report = ok_of(
        service
            .complete_workflow_task(
                &QianjiBpmnWorkflowTaskCompleteRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    instance_id: "wf_task_complete_action".to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                    completion: QianjiBpmnWorkflowTaskCompletionPayload {
                        token_id: pending_token_id,
                        process_id: "review".to_string(),
                        activity_id: pending_activity_id,
                        kind: QianjiBpmnWorkflowTaskCompletionKind::User,
                        data: json!({
                            "approved": true,
                            "source": "workflow_control_task_complete",
                        }),
                        claimant: None,
                    },
                    continue_until_human_boundary: false,
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "workflow control service should complete one pending host task",
    );

    assert_eq!(
        complete_report.execution.outcome,
        BpmnAdvanceOutcome::Completed
    );
    assert!(complete_report.execution.resumed_from_checkpoint);
    assert!(complete_report.execution.checkpoint_saved);
    assert_eq!(
        complete_report.execution.session.instance().variables,
        json!({
            "risk": "high",
            "approved": true,
            "source": "workflow_control_task_complete",
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_can_continue_to_next_human_boundary() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_service_user_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-human-boundary.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let (pending_token_id, pending_activity_id) = seed_pending_user_task_checkpoint_with_instance(
        &bpmn_path,
        &duckdb_path,
        "wf_task_complete_human_boundary",
    )
    .await;
    let host = QianjiBpmnHostBridge::builder()
        .on_service_task(|_request| async move {
            Ok(ServiceTaskOutcome {
                data: json!({
                    "stored": true,
                }),
            })
        })
        .build();

    let complete_report = ok_of(
        service
            .complete_workflow_task(
                &QianjiBpmnWorkflowTaskCompleteRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    instance_id: "wf_task_complete_human_boundary".to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                    completion: QianjiBpmnWorkflowTaskCompletionPayload {
                        token_id: pending_token_id,
                        process_id: "review".to_string(),
                        activity_id: pending_activity_id,
                        kind: QianjiBpmnWorkflowTaskCompletionKind::User,
                        data: json!({
                            "firstAnswer": "ready",
                        }),
                        claimant: None,
                    },
                    continue_until_human_boundary: true,
                },
                &host,
            )
            .await,
        "workflow control service should continue through service work to next user task",
    );

    assert!(matches!(
        &complete_report.execution.outcome,
        BpmnAdvanceOutcome::BlockedOnHost(pending)
            if pending.len() == 1 && pending[0].kind == PendingHostWorkKind::User
    ));
    assert!(complete_report.execution.resumed_from_checkpoint);
    assert!(complete_report.execution.checkpoint_saved);
    assert_eq!(
        complete_report.execution.session.instance().variables,
        json!({
            "risk": "high",
            "firstAnswer": "ready",
            "stored": true,
        })
    );
    assert_eq!(
        complete_report
            .execution
            .session
            .instance()
            .pending_host_work[0]
            .activity_id
            .as_deref(),
        Some("second_user")
    );
}
