#![cfg(feature = "duckdb")]

pub(super) use super::support::{
    BpmnAdvanceOutcome, QianjiBpmnHostBridge, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowTaskCompleteRequest,
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
    QianjiRuntimeEnv, TempDir, json, ok_of, write_form_user_task_bundle,
    write_user_service_user_bundle, write_user_task_bundle,
};
pub(super) use crate::{QianjiBpmnCheckpointStore, load_bpmn_package_from_files};
pub(super) use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnInstanceInit, PendingHostWorkKind, PendingHostWorkResult,
    ServiceTaskOutcome, UserTaskOutcome, advance_instance, apply_pending_host_work_result,
    create_instance,
};

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
                    "answer": "ready",
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

mod claims;
mod completion;
mod prepared;
mod validation;
