use super::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload, QianjiRuntimeEnv, TempDir, json,
    seed_pending_user_task_checkpoint, write_form_user_task_bundle, write_user_task_bundle,
};

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_rejects_undeclared_form_field() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_form_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-form-extra.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path).await;

    let error = match service
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
                        "answer": "approve",
                        "source": "undeclared",
                    }),
                    claimant: None,
                },
                continue_until_human_boundary: false,
            },
            &QianjiBpmnHostBridge::default(),
        )
        .await
    {
        Ok(report) => panic!("undeclared form output should fail, got {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("contains undeclared output 'source'")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_rejects_non_object_form_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_form_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-form-non-object.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path).await;

    let error = match service
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
                    data: json!("approve"),
                    claimant: None,
                },
                continue_until_human_boundary: false,
            },
            &QianjiBpmnHostBridge::default(),
        )
        .await
    {
        Ok(report) => panic!("non-object form output should fail, got {report:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("must be a JSON object"));
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_rejects_nested_form_output_envelope() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_form_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-form-nested.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path).await;

    let error = match service
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
                        "form": {
                            "answer": "approve",
                            "feedback": "nested envelope",
                        },
                    }),
                    claimant: None,
                },
                continue_until_human_boundary: false,
            },
            &QianjiBpmnHostBridge::default(),
        )
        .await
    {
        Ok(report) => panic!("nested form output envelope should fail, got {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing declared output 'answer'")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_rejects_activity_identity_mismatch() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir
        .path()
        .join("task-complete-identity-mismatch.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let (pending_token_id, _pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path).await;

    let error = match service
        .complete_workflow_task(
            &QianjiBpmnWorkflowTaskCompleteRequest {
                bpmn_path,
                dmn_paths: Vec::new(),
                instance_id: "wf_task_complete_action".to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                completion: QianjiBpmnWorkflowTaskCompletionPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string(),
                    activity_id: "different_task".to_string(),
                    kind: QianjiBpmnWorkflowTaskCompletionKind::User,
                    data: json!({
                        "approved": true,
                    }),
                    claimant: None,
                },
                continue_until_human_boundary: false,
            },
            &QianjiBpmnHostBridge::default(),
        )
        .await
    {
        Ok(report) => panic!("identity mismatch should fail, got {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("pending host work identity mismatch")
    );
}
