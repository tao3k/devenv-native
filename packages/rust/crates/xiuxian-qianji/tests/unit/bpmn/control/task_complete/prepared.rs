use super::{
    BpmnAdvanceOutcome, PendingHostWorkKind, QianjiBpmnHostBridge,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowStartRequest,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload, QianjiRuntimeEnv, TempDir, json, ok_of,
    seed_pending_service_task_checkpoint_with_instance,
    seed_pending_user_task_checkpoint_with_instance, write_user_service_user_bundle,
    write_user_task_bundle,
};

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_prepared_task_complete_can_stop_at_next_host_boundary() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_service_user_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-host-boundary.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);
    let instance_id = "wf_task_complete_host_boundary";

    let (pending_token_id, pending_activity_id) =
        seed_pending_service_task_checkpoint_with_instance(&bpmn_path, &duckdb_path, instance_id)
            .await;
    let prepared_start = ok_of(
        service.prepare_start_workflow(&QianjiBpmnWorkflowStartRequest {
            bpmn_path: bpmn_path.clone(),
            dmn_paths: Vec::new(),
            process_id: "review".to_string(),
            instance_id: instance_id.to_string(),
            initial_variables: Some(json!({})),
            start_at_node_id: None,
            checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb),
        }),
        "workflow control service should prepare a reusable start package",
    );
    let prepared_resume = ok_of(
        service
            .prepare_resume_workflow_from_prepared_start(
                &QianjiBpmnWorkflowResumeRequest {
                    bpmn_path: bpmn_path.clone(),
                    dmn_paths: Vec::new(),
                    instance_id: instance_id.to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                },
                &prepared_start,
            )
            .await,
        "workflow control service should prepare resume from reusable start package",
    );

    let complete_report = ok_of(
        service
            .complete_prepared_workflow_task_until_host_boundary(
                prepared_resume,
                &QianjiBpmnWorkflowTaskCompleteRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    instance_id: instance_id.to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                    completion: QianjiBpmnWorkflowTaskCompletionPayload {
                        token_id: pending_token_id,
                        process_id: "review".to_string(),
                        activity_id: pending_activity_id,
                        kind: QianjiBpmnWorkflowTaskCompletionKind::Service,
                        data: json!({
                            "stored": true,
                        }),
                        claimant: None,
                    },
                    continue_until_human_boundary: false,
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "prepared completion should expose the next host work without dispatching it",
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

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_prepared_task_complete_reuses_prepared_package() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-prepared-reuse.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);
    let instance_id = "wf_task_complete_prepared_reuse";

    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint_with_instance(&bpmn_path, &duckdb_path, instance_id)
            .await;
    let prepared_start = ok_of(
        service.prepare_start_workflow(&QianjiBpmnWorkflowStartRequest {
            bpmn_path: bpmn_path.clone(),
            dmn_paths: Vec::new(),
            process_id: "review".to_string(),
            instance_id: instance_id.to_string(),
            initial_variables: Some(json!({})),
            start_at_node_id: None,
            checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb),
        }),
        "workflow control service should prepare a reusable start package",
    );
    let prepared_resume = ok_of(
        service
            .prepare_resume_workflow_from_prepared_start(
                &QianjiBpmnWorkflowResumeRequest {
                    bpmn_path: bpmn_path.clone(),
                    dmn_paths: Vec::new(),
                    instance_id: instance_id.to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                },
                &prepared_start,
            )
            .await,
        "workflow control service should prepare resume from reusable start package",
    );

    assert!(std::sync::Arc::ptr_eq(
        &prepared_start.package,
        &prepared_resume.package
    ));
    assert!(prepared_resume.loaded_checkpoint.is_some());
    assert_eq!(
        prepared_resume.resolved_bpmn_path,
        prepared_start.resolved_bpmn_path
    );
    let checkpoint_store = prepared_resume
        .checkpoint_store
        .clone()
        .unwrap_or_else(|| panic!("prepared resume should keep a checkpoint store"));
    ok_of(
        checkpoint_store.delete(instance_id).await,
        "prepared reuse proof should remove persisted checkpoint before completion",
    );

    let complete_report = ok_of(
        service
            .complete_prepared_workflow_task(
                prepared_resume,
                &QianjiBpmnWorkflowTaskCompleteRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    instance_id: instance_id.to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                    completion: QianjiBpmnWorkflowTaskCompletionPayload {
                        token_id: pending_token_id,
                        process_id: "review".to_string(),
                        activity_id: pending_activity_id,
                        kind: QianjiBpmnWorkflowTaskCompletionKind::User,
                        data: json!({
                            "answer": "prepared",
                        }),
                        claimant: None,
                    },
                    continue_until_human_boundary: false,
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "workflow control service should complete from prepared resume",
    );

    assert_eq!(
        complete_report.execution.outcome,
        BpmnAdvanceOutcome::Completed
    );
    assert_eq!(
        complete_report.execution.session.instance().variables,
        json!({
            "risk": "high",
            "answer": "prepared",
        })
    );
}
