#![cfg(feature = "duckdb")]

use super::support::*;
use crate::{QianjiBpmnCheckpointStore, load_bpmn_package_from_files};
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnInstanceInit, PendingHostWorkKind, ServiceTaskOutcome,
    advance_instance, create_instance,
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
    let pending_activity_id = instance.pending_host_work[0]
        .activity_id
        .clone()
        .expect("pending host work should carry activity id");
    let store = QianjiBpmnCheckpointStore::duckdb(duckdb_path);
    ok_of(
        store
            .save(&BpmnCheckpointEnvelope::from_state(instance))
            .await,
        "pending service task checkpoint should persist",
    );
    (pending_token_id, pending_activity_id)
}
