#![cfg(feature = "duckdb")]

use super::support::{
    BpmnAdvanceOutcome, QianjiBpmnHostBridge, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowTaskClaimPayload,
    QianjiBpmnWorkflowTaskClaimRequest, QianjiBpmnWorkflowTaskCompleteRequest,
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
    QianjiBpmnWorkflowTaskReleasePayload, QianjiBpmnWorkflowTaskReleaseRequest,
    QianjiBpmnWorkflowWorklistRequest, QianjiBpmnWorkflowWorklistRoutingFilter, QianjiRuntimeEnv,
    TempDir, json, ok_of, write_assignment_user_task_bundle, write_lane_user_task_bundle,
    write_user_task_bundle,
};
use crate::{
    QianjiBpmnCheckpointStore, QianjiBpmnWorkflowStatusRequest, QianjiBpmnWorkflowWorklistItem,
    load_bpmn_package_from_files,
};
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnHumanTaskLifecycleEventKind, BpmnInstanceInit, PendingHostWork,
    PendingHostWorkClaim, advance_instance, create_instance,
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
                instance_id: instance_id.to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string().into(),
                    activity_id: pending_activity_id.into(),
                    claimant: "alice".to_string(),
                },
            })
            .await,
        "workflow control service should claim one pending human task",
    );

    assert!(claim_report.changed);
    assert_human_task_event_kinds(
        &claim_report.instance,
        &[
            BpmnHumanTaskLifecycleEventKind::Created,
            BpmnHumanTaskLifecycleEventKind::Claimed,
        ],
    );
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
                instance_id: instance_id.to_string().into(),
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
    assert_human_task_event_kinds(
        &status_report.instance,
        &[
            BpmnHumanTaskLifecycleEventKind::Created,
            BpmnHumanTaskLifecycleEventKind::Claimed,
        ],
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
                routing: QianjiBpmnWorkflowWorklistRoutingFilter::default(),
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
                instance_id: instance_id.to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string().into(),
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
                routing: QianjiBpmnWorkflowWorklistRoutingFilter::default(),
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
                routing: QianjiBpmnWorkflowWorklistRoutingFilter::default(),
            })
            .await,
        "different claimant worklist should load",
    );
    assert!(bob_work.work_items.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_worklist_filters_assignment_routing_metadata_without_authorization()
 {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_assignment_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-worklist-routing.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);
    let instance_id = "wf_task_worklist_routing";
    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path, instance_id).await;

    let reviewer_work = ok_of(
        service
            .list_workflow_worklist(&worklist_request(None, Some("reviewer")))
            .await,
        "worklist should filter by humanPerformer name",
    );
    assert_eq!(reviewer_work.work_items.len(), 1);
    assert_eq!(reviewer_work.work_items[0].instance_id, instance_id);
    assert_eq!(reviewer_work.work_items[0].token_id, pending_token_id);
    assert_eq!(reviewer_work.work_items[0].activity_id, pending_activity_id);

    let review_team_work = ok_of(
        service
            .list_workflow_worklist(&worklist_request(None, Some("reviewers")))
            .await,
        "worklist should filter by potentialOwner resourceRef",
    );
    assert_eq!(review_team_work.work_items.len(), 1);

    let finance_work = ok_of(
        service
            .list_workflow_worklist(&worklist_request(None, Some("finance")))
            .await,
        "non-matching assignment resource filter should load",
    );
    assert!(finance_work.work_items.is_empty());

    let claimed_by_non_resource = ok_of(
        service
            .claim_workflow_task(&QianjiBpmnWorkflowTaskClaimRequest {
                instance_id: instance_id.to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string().into(),
                    activity_id: review_team_work.work_items[0].activity_id.clone(),
                    claimant: "finance-user".to_string(),
                },
            })
            .await,
        "assignment routing filter must not authorize claim",
    );
    assert_eq!(
        claimed_by_non_resource
            .claimed_work
            .claim
            .as_ref()
            .map(|claim| claim.claimant.as_str()),
        Some("finance-user")
    );

    let claimant_and_assignment = ok_of(
        service
            .list_workflow_worklist(&worklist_request(Some("finance-user"), Some("reviewers")))
            .await,
        "claimant and assignment routing filters should compose",
    );
    assert_eq!(claimant_and_assignment.work_items.len(), 1);

    let hidden_by_claimant = ok_of(
        service
            .list_workflow_worklist(&worklist_request(Some("alice"), Some("reviewers")))
            .await,
        "assignment routing must still respect claimant filtering",
    );
    assert!(hidden_by_claimant.work_items.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_worklist_filters_lane_metadata_without_authorization() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_lane_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-worklist-lane.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);
    let instance_id = "wf_task_worklist_lane";
    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path, instance_id).await;

    let lane_name_work = ok_of(
        service
            .list_workflow_worklist(&worklist_request_with_lane(
                None,
                None,
                Some("Reviewer Lane"),
            ))
            .await,
        "worklist should filter by lane name",
    );
    assert_eq!(lane_name_work.work_items.len(), 1);
    assert_lane_work_item(
        &lane_name_work.work_items[0],
        instance_id,
        pending_token_id,
        &pending_activity_id,
    );

    let lane_id_work = ok_of(
        service
            .list_workflow_worklist(&worklist_request_with_lane(
                None,
                Some("reviewers"),
                Some("Lane_Reviewer"),
            ))
            .await,
        "worklist should compose assignment resource and lane id filters",
    );
    assert_eq!(lane_id_work.work_items.len(), 1);

    let hidden_by_lane = ok_of(
        service
            .list_workflow_worklist(&worklist_request_with_lane(
                None,
                None,
                Some("Finance Lane"),
            ))
            .await,
        "non-matching lane filter should load",
    );
    assert!(hidden_by_lane.work_items.is_empty());

    let claimed_by_non_lane_actor = ok_of(
        service
            .claim_workflow_task(&QianjiBpmnWorkflowTaskClaimRequest {
                instance_id: instance_id.to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string().into(),
                    activity_id: lane_name_work.work_items[0].activity_id.clone(),
                    claimant: "finance-user".to_string(),
                },
            })
            .await,
        "lane routing filter must not authorize claim",
    );
    assert_eq!(
        claimed_by_non_lane_actor
            .claimed_work
            .claim
            .as_ref()
            .map(|claim| claim.claimant.as_str()),
        Some("finance-user")
    );

    let claimant_and_lane = ok_of(
        service
            .list_workflow_worklist(&worklist_request_with_lane(
                Some("finance-user"),
                None,
                Some("Reviewer Lane"),
            ))
            .await,
        "claimant and lane filters should compose",
    );
    assert_eq!(claimant_and_lane.work_items.len(), 1);

    let hidden_by_claimant = ok_of(
        service
            .list_workflow_worklist(&worklist_request_with_lane(
                Some("alice"),
                None,
                Some("Reviewer Lane"),
            ))
            .await,
        "lane routing must still respect claimant filtering",
    );
    assert!(hidden_by_claimant.work_items.is_empty());
}

fn assert_lane_work_item(
    item: &QianjiBpmnWorkflowWorklistItem,
    instance_id: &str,
    token_id: u64,
    activity_id: &str,
) {
    assert_eq!(item.instance_id, instance_id);
    assert_eq!(item.token_id, token_id);
    assert_eq!(item.activity_id, activity_id);
    assert_eq!(
        item.lane.as_ref().and_then(|lane| lane.id.as_deref()),
        Some("Lane_Reviewer")
    );
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
                instance_id: instance_id.to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string().into(),
                    activity_id: pending_activity_id.clone().into(),
                    claimant: "alice".to_string(),
                },
            })
            .await,
        "human task should be claimed before completion validation",
    );

    let build_completion = |claimant: Option<&str>| QianjiBpmnWorkflowTaskCompleteRequest {
        bpmn_path: bpmn_path.clone(),
        dmn_paths: Vec::new(),
        instance_id: instance_id.to_string().into(),
        checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
        completion: QianjiBpmnWorkflowTaskCompletionPayload {
            token_id: pending_token_id,
            process_id: "review".to_string().into(),
            activity_id: pending_activity_id.clone().into(),
            kind: QianjiBpmnWorkflowTaskCompletionKind::User,
            data: json!({
                "answer": "approved",
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
                instance_id: instance_id.to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claim: QianjiBpmnWorkflowTaskClaimPayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string().into(),
                    activity_id: pending_activity_id.clone().into(),
                    claimant: "alice".to_string(),
                },
            })
            .await,
        "human task should be claimed before release validation",
    );

    let mismatch = match service
        .release_workflow_task(&QianjiBpmnWorkflowTaskReleaseRequest {
            instance_id: instance_id.to_string().into(),
            checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            release: QianjiBpmnWorkflowTaskReleasePayload {
                token_id: pending_token_id,
                process_id: "review".to_string().into(),
                activity_id: pending_activity_id.clone().into(),
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
                instance_id: instance_id.to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                release: QianjiBpmnWorkflowTaskReleasePayload {
                    token_id: pending_token_id,
                    process_id: "review".to_string().into(),
                    activity_id: pending_activity_id.into(),
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
                routing: QianjiBpmnWorkflowWorklistRoutingFilter::default(),
            })
            .await,
        "released work should appear as unclaimed for claimant-filtered worklist",
    );
    assert_eq!(alice_work.work_items.len(), 1);
    assert!(alice_work.work_items[0].claim.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_preserves_claim_identity_across_checkpoint_roundtrip() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-claim-identity-roundtrip.duckdb");
    let instance_id = "wf_task_claim_identity_roundtrip";
    let (pending_token_id, pending_activity_id) =
        seed_pending_user_task_checkpoint(&bpmn_path, &duckdb_path, instance_id).await;
    let identity = HumanTaskIdentity::new(instance_id, pending_token_id, pending_activity_id);

    let initial_sequence = assert_initial_identity_checkpoint(&duckdb_path, &identity).await;
    let claim_sequence = claim_identity(&duckdb_path, &identity, "alice", initial_sequence).await;
    assert_claimed_identity_checkpoint(&duckdb_path, &identity, "alice").await;
    let release_sequence = release_identity(&duckdb_path, &identity, "alice", claim_sequence).await;
    assert_released_worklist_identity(&duckdb_path, &identity, "alice").await;
    claim_identity(&duckdb_path, &identity, "alice", release_sequence).await;
    assert_wrong_claimant_completion_fails(&duckdb_path, &bpmn_path, &identity, "bob").await;
    complete_same_claimant(&duckdb_path, &bpmn_path, &identity, "alice").await;
}

fn assert_human_task_event_kinds(
    instance: &qianji_bpmn_engine::BpmnInstanceState,
    expected: &[BpmnHumanTaskLifecycleEventKind],
) {
    let actual = instance
        .human_task_events
        .iter()
        .map(|event| event.kind.clone())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

struct HumanTaskIdentity {
    instance: String,
    token: u64,
    activity: String,
}

impl HumanTaskIdentity {
    const PROCESS_ID: &'static str = "review";

    fn new(instance_id: &str, token_id: u64, activity_id: String) -> Self {
        Self {
            instance: instance_id.to_string(),
            token: token_id,
            activity: activity_id,
        }
    }

    fn assert_pending(&self, pending: &PendingHostWork) {
        assert_eq!(pending.token_id, self.token);
        assert_eq!(pending.process_id.as_deref(), Some(Self::PROCESS_ID));
        assert_eq!(pending.activity_id.as_deref(), Some(self.activity.as_str()));
    }

    fn status_request(&self) -> QianjiBpmnWorkflowStatusRequest {
        QianjiBpmnWorkflowStatusRequest {
            instance_id: self.instance.clone().into(),
            checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
        }
    }

    fn claim_request(&self, claimant: &str) -> QianjiBpmnWorkflowTaskClaimRequest {
        QianjiBpmnWorkflowTaskClaimRequest {
            instance_id: self.instance.clone().into(),
            checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            claim: QianjiBpmnWorkflowTaskClaimPayload {
                token_id: self.token,
                process_id: Self::PROCESS_ID.to_string().into(),
                activity_id: self.activity.clone().into(),
                claimant: claimant.to_string(),
            },
        }
    }

    fn release_request(&self, claimant: &str) -> QianjiBpmnWorkflowTaskReleaseRequest {
        QianjiBpmnWorkflowTaskReleaseRequest {
            instance_id: self.instance.clone().into(),
            checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            release: QianjiBpmnWorkflowTaskReleasePayload {
                token_id: self.token,
                process_id: Self::PROCESS_ID.to_string().into(),
                activity_id: self.activity.clone().into(),
                claimant: claimant.to_string(),
            },
        }
    }

    fn completion_request(
        &self,
        bpmn_path: &std::path::Path,
        claimant: &str,
    ) -> QianjiBpmnWorkflowTaskCompleteRequest {
        QianjiBpmnWorkflowTaskCompleteRequest {
            bpmn_path: bpmn_path.to_path_buf(),
            dmn_paths: Vec::new(),
            instance_id: self.instance.clone().into(),
            checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            completion: QianjiBpmnWorkflowTaskCompletionPayload {
                token_id: self.token,
                process_id: Self::PROCESS_ID.to_string().into(),
                activity_id: self.activity.clone().into(),
                kind: QianjiBpmnWorkflowTaskCompletionKind::User,
                data: json!({
                    "answer": "claim_identity_roundtrip",
                }),
                claimant: Some(claimant.to_string()),
            },
            continue_until_human_boundary: false,
        }
    }
}

async fn assert_initial_identity_checkpoint(
    duckdb_path: &std::path::Path,
    identity: &HumanTaskIdentity,
) -> u64 {
    let service = new_control_service(duckdb_path);
    let status = ok_of(
        service
            .load_workflow_status(&identity.status_request())
            .await,
        "initial human task status should load from checkpoint",
    );
    assert_eq!(
        status.instance.instance_id.as_ref(),
        identity.instance.as_str()
    );
    assert_eq!(status.instance.pending_host_work.len(), 1);
    identity.assert_pending(&status.instance.pending_host_work[0]);
    assert!(status.instance.pending_host_work[0].claim.is_none());
    assert_human_task_event_kinds(
        &status.instance,
        &[BpmnHumanTaskLifecycleEventKind::Created],
    );
    status.checkpoint_sequence
}

async fn claim_identity(
    duckdb_path: &std::path::Path,
    identity: &HumanTaskIdentity,
    claimant: &str,
    previous_sequence: u64,
) -> u64 {
    let service = new_control_service(duckdb_path);
    let report = ok_of(
        service
            .claim_workflow_task(&identity.claim_request(claimant))
            .await,
        "fresh service should claim checkpointed human task",
    );
    assert!(report.changed);
    identity.assert_pending(&report.claimed_work);
    assert_eq!(
        report
            .claimed_work
            .claim
            .as_ref()
            .map(|claim| claim.claimant.as_str()),
        Some(claimant)
    );
    assert_eq!(
        report
            .instance
            .human_task_events
            .last()
            .map(|event| (&event.kind, event.claimant.as_deref())),
        Some((&BpmnHumanTaskLifecycleEventKind::Claimed, Some(claimant)))
    );
    assert!(report.checkpoint_sequence > previous_sequence);
    report.checkpoint_sequence
}

async fn assert_claimed_identity_checkpoint(
    duckdb_path: &std::path::Path,
    identity: &HumanTaskIdentity,
    claimant: &str,
) {
    let service = new_control_service(duckdb_path);
    let status = ok_of(
        service
            .load_workflow_status(&identity.status_request())
            .await,
        "claimed status should reload from checkpoint",
    );
    identity.assert_pending(&status.instance.pending_host_work[0]);
    assert_eq!(
        status.instance.pending_host_work[0]
            .claim
            .as_ref()
            .map(|claim| claim.claimant.as_str()),
        Some(claimant)
    );
    assert_eq!(
        status
            .instance
            .human_task_events
            .last()
            .map(|event| (&event.kind, event.claimant.as_deref())),
        Some((&BpmnHumanTaskLifecycleEventKind::Claimed, Some(claimant)))
    );
}

async fn release_identity(
    duckdb_path: &std::path::Path,
    identity: &HumanTaskIdentity,
    claimant: &str,
    previous_sequence: u64,
) -> u64 {
    let service = new_control_service(duckdb_path);
    let report = ok_of(
        service
            .release_workflow_task(&identity.release_request(claimant))
            .await,
        "fresh service should release checkpointed human task",
    );
    assert!(report.changed);
    identity.assert_pending(&report.released_work);
    assert!(report.released_work.claim.is_none());
    assert_eq!(
        report
            .instance
            .human_task_events
            .last()
            .map(|event| (&event.kind, event.claimant.as_deref())),
        Some((&BpmnHumanTaskLifecycleEventKind::Released, Some(claimant)))
    );
    assert!(report.checkpoint_sequence > previous_sequence);
    report.checkpoint_sequence
}

async fn assert_released_worklist_identity(
    duckdb_path: &std::path::Path,
    identity: &HumanTaskIdentity,
    claimant: &str,
) {
    let service = new_control_service(duckdb_path);
    let worklist = ok_of(
        service
            .list_workflow_worklist(&QianjiBpmnWorkflowWorklistRequest {
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                claimant: Some(claimant.to_string()),
                routing: QianjiBpmnWorkflowWorklistRoutingFilter::default(),
            })
            .await,
        "released work should reload into claimant-visible worklist",
    );
    assert_eq!(worklist.work_items.len(), 1);
    assert_eq!(
        worklist.work_items[0].instance_id.as_str(),
        identity.instance.as_str()
    );
    assert_eq!(worklist.work_items[0].token_id, identity.token);
    assert_eq!(
        worklist.work_items[0].process_id.as_str(),
        HumanTaskIdentity::PROCESS_ID
    );
    assert_eq!(
        worklist.work_items[0].activity_id.as_str(),
        identity.activity.as_str()
    );
    assert!(worklist.work_items[0].claim.is_none());
}

async fn assert_wrong_claimant_completion_fails(
    duckdb_path: &std::path::Path,
    bpmn_path: &std::path::Path,
    identity: &HumanTaskIdentity,
    claimant: &str,
) {
    let service = new_control_service(duckdb_path);
    let error = match service
        .complete_workflow_task(
            &identity.completion_request(bpmn_path, claimant),
            &QianjiBpmnHostBridge::default(),
        )
        .await
    {
        Ok(report) => panic!("wrong claimant completion should fail, got {report:?}"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("expected claimant 'alice', got 'bob'")
    );
    let status = ok_of(
        service
            .load_workflow_status(&identity.status_request())
            .await,
        "status should still load after rejected wrong-claimant completion",
    );
    assert_eq!(
        status
            .instance
            .human_task_events
            .last()
            .map(|event| event.kind.clone()),
        Some(BpmnHumanTaskLifecycleEventKind::Claimed)
    );
}

async fn complete_same_claimant(
    duckdb_path: &std::path::Path,
    bpmn_path: &std::path::Path,
    identity: &HumanTaskIdentity,
    claimant: &str,
) {
    let service = new_control_service(duckdb_path);
    let report = ok_of(
        service
            .complete_workflow_task(
                &identity.completion_request(bpmn_path, claimant),
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "same claimant should complete checkpointed human task",
    );
    assert_eq!(report.execution.outcome, BpmnAdvanceOutcome::Completed);
    assert!(report.execution.resumed_from_checkpoint);
    assert_eq!(
        report
            .execution
            .session
            .instance()
            .human_task_events
            .last()
            .map(|event| (&event.kind, event.claimant.as_deref())),
        Some((&BpmnHumanTaskLifecycleEventKind::Completed, Some(claimant)))
    );
    assert_eq!(
        report.execution.session.instance().variables,
        json!({
            "risk": "high",
            "answer": "claim_identity_roundtrip",
        })
    );
}

fn new_control_service(duckdb_path: &std::path::Path) -> QianjiBpmnWorkflowControlService {
    QianjiBpmnWorkflowControlService::new().with_runtime_env(QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.to_path_buf()),
        ..QianjiRuntimeEnv::default()
    })
}

fn worklist_request(
    claimant: Option<&str>,
    assignment_resource: Option<&str>,
) -> QianjiBpmnWorkflowWorklistRequest {
    worklist_request_with_lane(claimant, assignment_resource, None)
}

fn worklist_request_with_lane(
    claimant: Option<&str>,
    assignment_resource: Option<&str>,
    lane: Option<&str>,
) -> QianjiBpmnWorkflowWorklistRequest {
    QianjiBpmnWorkflowWorklistRequest {
        checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
        claimant: claimant.map(str::to_string),
        routing: QianjiBpmnWorkflowWorklistRoutingFilter {
            assignment_resource: assignment_resource.map(str::to_string),
            lane: lane.map(str::to_string),
        },
    }
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
    (pending_token_id, pending_activity_id.as_str().to_string())
}
