use super::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload, QianjiRuntimeEnv, TempDir, json, ok_of,
    seed_pending_user_task_checkpoint, write_form_user_task_bundle,
};

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_accepts_declared_form_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_form_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-form-action.duckdb");
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
                    instance_id: "wf_task_complete_action".to_string().into(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                    completion: QianjiBpmnWorkflowTaskCompletionPayload {
                        token_id: pending_token_id,
                        process_id: "review".to_string().into(),
                        activity_id: pending_activity_id.into(),
                        kind: QianjiBpmnWorkflowTaskCompletionKind::User,
                        data: json!({
                            "answer": "approve",
                        }),
                        claimant: None,
                    },
                    continue_until_human_boundary: false,
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "form-backed workflow task completion should accept declared fields",
    );

    assert_eq!(
        complete_report.execution.session.instance().variables,
        json!({
            "risk": "high",
            "answer": "approve",
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_accepts_declared_result_without_optional_free_text()
{
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_form_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir
        .path()
        .join("task-complete-form-optional-omitted.duckdb");
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
                    instance_id: "wf_task_complete_action".to_string().into(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                    completion: QianjiBpmnWorkflowTaskCompletionPayload {
                        token_id: pending_token_id,
                        process_id: "review".to_string().into(),
                        activity_id: pending_activity_id.into(),
                        kind: QianjiBpmnWorkflowTaskCompletionKind::User,
                        data: json!({
                            "answer": "approve",
                        }),
                        claimant: None,
                    },
                    continue_until_human_boundary: false,
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "form-backed workflow task completion should allow optional free text omission",
    );

    assert_eq!(
        complete_report.execution.session.instance().variables,
        json!({
            "risk": "high",
            "answer": "approve",
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_rejects_missing_form_result() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_form_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-form-missing.duckdb");
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
                instance_id: "wf_task_complete_action".to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                completion: QianjiBpmnWorkflowTaskCompletionPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string().into(),
                    activity_id: pending_activity_id.into(),
                    kind: QianjiBpmnWorkflowTaskCompletionKind::User,
                    data: json!({
                        "feedback": "missing answer",
                    }),
                    claimant: None,
                },
                continue_until_human_boundary: false,
            },
            &QianjiBpmnHostBridge::default(),
        )
        .await
    {
        Ok(report) => panic!("missing form output should fail, got {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing declared output 'answer'")
    );
}
