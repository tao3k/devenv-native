#![cfg(feature = "duckdb")]

use super::support::*;
use crate::{QianjiBpmnCheckpointStore, load_bpmn_package_from_files};
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnInstanceInit, PendingHostWorkKind, PendingHostWorkResult,
    ServiceTaskOutcome, UserTaskOutcome, advance_instance, apply_pending_host_work_result,
    create_instance,
};

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
                    instance_id: "wf_task_complete_action".to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                    completion: QianjiBpmnWorkflowTaskCompletionPayload {
                        token_id: pending_token_id,
                        process_id: "review".to_string(),
                        activity_id: pending_activity_id,
                        kind: QianjiBpmnWorkflowTaskCompletionKind::User,
                        data: json!({
                            "answer": "approve",
                            "feedback": "looks good",
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
            "feedback": "looks good",
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
                instance_id: "wf_task_complete_action".to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                completion: QianjiBpmnWorkflowTaskCompletionPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string(),
                    activity_id: pending_activity_id,
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
            .contains("missing required field 'answer'")
    );
}

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
            .contains("contains undeclared field 'source'")
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

async fn seed_pending_user_task_checkpoint(
    bpmn_path: &std::path::Path,
    duckdb_path: &std::path::Path,
) -> (u64, String) {
    seed_pending_user_task_checkpoint_with_instance(
        bpmn_path,
        duckdb_path,
        "wf_task_complete_action",
    )
    .await
}

async fn seed_pending_user_task_checkpoint_with_instance(
    bpmn_path: &std::path::Path,
    duckdb_path: &std::path::Path,
    instance_id: &str,
) -> (u64, String) {
    let package = ok_of(
        load_bpmn_package_from_files(bpmn_path, &[]),
        "user task package should load for checkpoint seed",
    );
    let mut instance = ok_of(
        create_instance(
            package.clone(),
            "review",
            BpmnInstanceInit::new(instance_id, json!({ "risk": "high" }), 10),
        ),
        "user task instance should seed",
    );
    let outcome = ok_of(
        advance_instance(
            package.as_ref(),
            &mut instance,
            &QianjiBpmnHostBridge::default(),
        )
        .await,
        "user task instance should block on pending host work",
    );

    assert!(matches!(outcome, BpmnAdvanceOutcome::BlockedOnHost(pending) if pending.len() == 1));
    assert_eq!(instance.pending_host_work.len(), 1);
    let pending_token_id = instance.pending_host_work[0].token_id;
    let pending_activity_id = match &instance.pending_host_work[0].activity_id {
        Some(activity_id) => activity_id.clone(),
        None => panic!("pending host work should carry activity id"),
    };
    let store = QianjiBpmnCheckpointStore::duckdb(duckdb_path);
    ok_of(
        store
            .save(&BpmnCheckpointEnvelope::from_state(instance))
            .await,
        "pending service task checkpoint should persist",
    );
    (pending_token_id, pending_activity_id)
}

async fn seed_pending_service_task_checkpoint_with_instance(
    bpmn_path: &std::path::Path,
    duckdb_path: &std::path::Path,
    instance_id: &str,
) -> (u64, String) {
    let package = ok_of(
        load_bpmn_package_from_files(bpmn_path, &[]),
        "service task package should load for checkpoint seed",
    );
    let mut instance = ok_of(
        create_instance(
            package.clone(),
            "review",
            BpmnInstanceInit::new(instance_id, json!({ "risk": "high" }), 10),
        ),
        "service task instance should seed",
    );
    let outcome = ok_of(
        advance_instance(
            package.as_ref(),
            &mut instance,
            &QianjiBpmnHostBridge::default(),
        )
        .await,
        "service task seed should first block on user work",
    );
    assert!(matches!(
        outcome,
        BpmnAdvanceOutcome::BlockedOnHost(pending)
            if pending.len() == 1 && pending[0].kind == PendingHostWorkKind::User
    ));
    let first_user_token_id = instance.pending_host_work[0].token_id;
    let completed_at_ms = instance.updated_at_ms;
    let mut outcome = ok_of(
        apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            first_user_token_id,
            PendingHostWorkResult::User(UserTaskOutcome {
                data: json!({
                    "firstAnswer": "ready",
                }),
            }),
            completed_at_ms,
        ),
        "service task seed should complete first user work",
    );
    loop {
        match outcome {
            BpmnAdvanceOutcome::Advanced => {
                outcome = ok_of(
                    advance_instance(
                        package.as_ref(),
                        &mut instance,
                        &QianjiBpmnHostBridge::default(),
                    )
                    .await,
                    "service task seed should advance toward service work",
                );
            }
            BpmnAdvanceOutcome::BlockedOnHost(pending) => {
                assert_eq!(pending.len(), 1);
                assert_eq!(pending[0].kind, PendingHostWorkKind::Service);
                break;
            }
            other => panic!("service task seed should stop at service host work, got {other:?}"),
        }
    }

    assert_eq!(instance.pending_host_work.len(), 1);
    let pending_token_id = instance.pending_host_work[0].token_id;
    let pending_activity_id = match &instance.pending_host_work[0].activity_id {
        Some(activity_id) => activity_id.clone(),
        None => panic!("pending host work should carry activity id"),
    };
    let store = QianjiBpmnCheckpointStore::duckdb(duckdb_path);
    ok_of(
        store
            .save(&BpmnCheckpointEnvelope::from_state(instance))
            .await,
        "pending service task checkpoint should persist",
    );
    (pending_token_id, pending_activity_id)
}
