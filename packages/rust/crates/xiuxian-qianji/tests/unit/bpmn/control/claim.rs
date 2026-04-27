#![cfg(feature = "duckdb")]

use super::support::*;
use crate::{
    QianjiBpmnCheckpointStore, QianjiBpmnWorkflowStatusRequest, load_bpmn_package_from_files,
};
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnInstanceInit, PendingHostWorkClaim, advance_instance,
    create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_claim_persists_human_task_owner() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-claim-action.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);
    let instance_id = "wf_task_claim_action";
    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path, instance_id).await;

    let claim_report = ok_of(
        service
            .claim_workflow_task(&QianjiBpmnWorkflowTaskClaimRequest {
                instance_id: instance_id.to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string(),
                    activity_id: pending_activity_id,
                    claimant: "alice".to_string(),
                },
            })
            .await,
        "workflow control service should claim one pending human task",
    );

    assert!(claim_report.changed);
    assert_eq!(
        claim_report
            .claimed_work
            .claim
            .as_ref()
            .map(|claim| claim.claimant.as_str()),
        Some("alice")
    );

    let status_report = ok_of(
        service
            .load_workflow_status(&QianjiBpmnWorkflowStatusRequest {
                instance_id: instance_id.to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            })
            .await,
        "claimed workflow status should load",
    );
    let claimed_at_ms = match &claim_report.claimed_work.claim {
        Some(claim) => claim.claimed_at_ms,
        None => panic!("claimed work should carry claim"),
    };
    assert_eq!(
        status_report.instance.pending_host_work[0].claim,
        Some(PendingHostWorkClaim {
            claimant: "alice".to_string(),
            claimed_at_ms,
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_worklist_filters_checkpointed_human_work() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-worklist.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);
    let instance_id = "wf_task_worklist";
    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path, instance_id).await;

    let all_work = ok_of(
        service
            .list_workflow_worklist(&QianjiBpmnWorkflowWorklistRequest {
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claimant: None,
            })
            .await,
        "worklist should enumerate checkpointed human work",
    );
    assert_eq!(all_work.work_items.len(), 1);
    assert_eq!(all_work.work_items[0].instance_id, instance_id);
    assert_eq!(all_work.work_items[0].token_id, pending_token_id);
    assert_eq!(all_work.work_items[0].activity_id, pending_activity_id);
    assert!(all_work.work_items[0].claim.is_none());

    ok_of(
        service
            .claim_workflow_task(&QianjiBpmnWorkflowTaskClaimRequest {
                instance_id: instance_id.to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string(),
                    activity_id: all_work.work_items[0].activity_id.clone(),
                    claimant: "alice".to_string(),
                },
            })
            .await,
        "worklist item should be claimable",
    );

    let alice_work = ok_of(
        service
            .list_workflow_worklist(&QianjiBpmnWorkflowWorklistRequest {
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claimant: Some("alice".to_string()),
            })
            .await,
        "claimant worklist should include matching claim",
    );
    assert_eq!(alice_work.work_items.len(), 1);
    assert_eq!(
        alice_work.work_items[0]
            .claim
            .as_ref()
            .map(|claim| claim.claimant.as_str()),
        Some("alice")
    );

    let bob_work = ok_of(
        service
            .list_workflow_worklist(&QianjiBpmnWorkflowWorklistRequest {
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claimant: Some("bob".to_string()),
            })
            .await,
        "different claimant worklist should load",
    );
    assert!(bob_work.work_items.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_rejects_claimed_task_completion_by_different_claimant() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-claim-completion.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);
    let instance_id = "wf_task_claim_completion";
    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path, instance_id).await;

    ok_of(
        service
            .claim_workflow_task(&QianjiBpmnWorkflowTaskClaimRequest {
                instance_id: instance_id.to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string(),
                    activity_id: pending_activity_id.clone(),
                    claimant: "alice".to_string(),
                },
            })
            .await,
        "human task should be claimed before completion validation",
    );

    let build_completion = |claimant: Option<&str>| QianjiBpmnWorkflowTaskCompleteRequest {
        bpmn_path: bpmn_path.clone(),
        dmn_paths: Vec::new(),
        instance_id: instance_id.to_string(),
        checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
        completion: QianjiBpmnWorkflowTaskCompletionPayload {
            token_id: pending_token_id,
            process_id: "review".to_string(),
            activity_id: pending_activity_id.clone(),
            kind: QianjiBpmnWorkflowTaskCompletionKind::User,
            data: json!({
                "approved": true,
            }),
            claimant: claimant.map(str::to_string),
        },
        continue_until_human_boundary: false,
    };

    let missing_request = build_completion(None);
    let error = match service
        .complete_workflow_task(&missing_request, &QianjiBpmnHostBridge::default())
        .await
    {
        Ok(report) => {
            panic!("claimed task completion without claimant should fail, got {report:?}")
        }
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("completion must include the matching claimant")
    );

    let bob_request = build_completion(Some("bob"));
    let error = match service
        .complete_workflow_task(&bob_request, &QianjiBpmnHostBridge::default())
        .await
    {
        Ok(report) => {
            panic!("claimed task completion by different claimant should fail, got {report:?}")
        }
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("expected claimant 'alice', got 'bob'")
    );

    let alice_request = build_completion(Some("alice"));
    let report = ok_of(
        service
            .complete_workflow_task(&alice_request, &QianjiBpmnHostBridge::default())
            .await,
        "matching claimant should complete claimed human task",
    );
    assert_eq!(report.execution.outcome, BpmnAdvanceOutcome::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_release_returns_human_task_to_unclaimed_worklist() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-claim-release.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);
    let instance_id = "wf_task_claim_release";
    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path, instance_id).await;

    ok_of(
        service
            .claim_workflow_task(&QianjiBpmnWorkflowTaskClaimRequest {
                instance_id: instance_id.to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string(),
                    activity_id: pending_activity_id.clone(),
                    claimant: "alice".to_string(),
                },
            })
            .await,
        "human task should be claimed before release validation",
    );

    let mismatch = match service
        .release_workflow_task(&QianjiBpmnWorkflowTaskReleaseRequest {
            instance_id: instance_id.to_string(),
            checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            release: QianjiBpmnWorkflowTaskReleasePayload {
                token_id: pending_token_id,
                process_id: "review".to_string(),
                activity_id: pending_activity_id.clone(),
                claimant: "bob".to_string(),
            },
        })
        .await
    {
        Ok(report) => panic!("different claimant release should fail, got {report:?}"),
        Err(error) => error,
    };
    assert!(mismatch.to_string().contains("cannot be released by 'bob'"));

    let release_report = ok_of(
        service
            .release_workflow_task(&QianjiBpmnWorkflowTaskReleaseRequest {
                instance_id: instance_id.to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                release: QianjiBpmnWorkflowTaskReleasePayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string(),
                    activity_id: pending_activity_id,
                    claimant: "alice".to_string(),
                },
            })
            .await,
        "same claimant should release human task claim",
    );
    assert!(release_report.changed);
    assert!(release_report.released_work.claim.is_none());
    assert!(release_report.instance.pending_host_work[0].claim.is_none());

    let alice_work = ok_of(
        service
            .list_workflow_worklist(&QianjiBpmnWorkflowWorklistRequest {
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claimant: Some("alice".to_string()),
            })
            .await,
        "released work should appear as unclaimed for claimant-filtered worklist",
    );
    assert_eq!(alice_work.work_items.len(), 1);
    assert!(alice_work.work_items[0].claim.is_none());
}

async fn seed_pending_user_task_checkpoint(
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
        "pending user task checkpoint should persist",
    );
    (pending_token_id, pending_activity_id)
}
