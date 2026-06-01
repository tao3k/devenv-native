use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowActionHttpRequest,
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowSnapshotHttpResponse,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskCompleteBatchHttpRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskReleaseHttpRequest,
};
#[cfg(feature = "valkey")]
use crate::{
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowHttpErrorBody, QianjiBpmnWorkflowHttpState,
    QianjiBpmnWorkflowRunHttpResponse, QianjiBpmnWorkflowStartRequest,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowTaskClaimHttpResponse,
    QianjiBpmnWorkflowTaskReleaseHttpResponse, SchedulerAgentIdentity, qianji_bpmn_workflow_router,
    runtime_config::QianjiRuntimeEnv,
};
#[cfg(feature = "valkey")]
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header::CONTENT_TYPE},
};
#[cfg(feature = "valkey")]
use serde::de::DeserializeOwned;
use serde_json::json;
use std::sync::Arc;
#[cfg(feature = "valkey")]
use std::{
    fs,
    path::{Path, PathBuf},
};
#[cfg(feature = "valkey")]
use tempfile::TempDir;
#[cfg(feature = "valkey")]
use tower::util::ServiceExt;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnHumanTaskAssignmentSpec, BpmnHumanTaskChoiceSpec,
    BpmnHumanTaskFormSpec, BpmnHumanTaskLifecycleEventKind, BpmnHumanTaskResourceRoleSpec,
    BpmnInstanceInit, BpmnLaneMembershipSpec, BpmnNodeKind, BpmnNodeSpec, BpmnPackage,
    BpmnProcessSpec, BpmnTaskInputBinding, BpmnTaskInputSource, BpmnTaskIoSpec,
    BpmnTaskOutputBinding, PendingHostWorkClaim, PendingHostWorkKind, PendingHumanTaskClaimInput,
    PendingHumanTaskClaimRequest, ProcessKey, advance_instance, claim_pending_human_task,
    create_instance,
};

#[cfg(feature = "valkey")]
use super::unique_instance_id;
#[cfg(feature = "valkey")]
use super::valkey_support::TestValkey;

#[path = "http/llm_completion_shape.rs"]
mod llm_completion_shape;
#[path = "http/llm_task_documentation.rs"]
mod llm_task_documentation;

#[test]
fn bpmn_workflow_http_requests_default_to_runtime_valkey_backend() {
    let start = serde_json::from_value::<QianjiBpmnWorkflowStartHttpRequest>(json!({
        "bpmn_path": "flow.bpmn",
        "process_id": "flow",
        "instance_id": "wf_http_default",
    }))
    .unwrap_or_else(|error| panic!("start HTTP request should decode: {error}"));
    assert_eq!(
        start.checkpoint_backend,
        QianjiBpmnWorkflowHttpCheckpointBackend::RuntimeValkey
    );

    let action = serde_json::from_value::<QianjiBpmnWorkflowActionHttpRequest>(json!({
        "bpmn_path": "flow.bpmn",
    }))
    .unwrap_or_else(|error| panic!("action HTTP request should decode: {error}"));
    assert_eq!(
        action.checkpoint_backend,
        QianjiBpmnWorkflowHttpCheckpointBackend::RuntimeValkey
    );

    let claim = serde_json::from_value::<QianjiBpmnWorkflowTaskClaimHttpRequest>(json!({
        "claim": {
            "token_id": 7,
            "process_id": "flow",
            "activity_id": "review",
            "claimant": "alice"
        }
    }))
    .unwrap_or_else(|error| panic!("claim HTTP request should decode: {error}"));
    assert_eq!(
        claim.checkpoint_backend,
        QianjiBpmnWorkflowHttpCheckpointBackend::RuntimeValkey
    );

    let release = serde_json::from_value::<QianjiBpmnWorkflowTaskReleaseHttpRequest>(json!({
        "release": {
            "token_id": 7,
            "process_id": "flow",
            "activity_id": "review",
            "claimant": "alice"
        }
    }))
    .unwrap_or_else(|error| panic!("release HTTP request should decode: {error}"));
    assert_eq!(
        release.checkpoint_backend,
        QianjiBpmnWorkflowHttpCheckpointBackend::RuntimeValkey
    );

    let task_complete =
        serde_json::from_value::<QianjiBpmnWorkflowTaskCompleteHttpRequest>(json!({
            "bpmn_path": "flow.bpmn",
            "completion": {
                "token_id": 7,
                "process_id": "flow",
                "activity_id": "review",
                "kind": "user",
                "data": {
                    "approved": true
                },
                "claimant": "alice"
            }
        }))
        .unwrap_or_else(|error| panic!("task-complete HTTP request should decode: {error}"));
    assert_eq!(
        task_complete.checkpoint_backend,
        QianjiBpmnWorkflowHttpCheckpointBackend::RuntimeValkey
    );
    assert_eq!(task_complete.completion.claimant.as_deref(), Some("alice"));

    let task_complete_batch =
        serde_json::from_value::<QianjiBpmnWorkflowTaskCompleteBatchHttpRequest>(json!({
            "bpmn_path": "flow.bpmn",
            "completions": [{
                "token_id": 7,
                "process_id": "flow",
                "activity_id": "review",
                "kind": "user",
                "data": {
                    "approved": true
                },
                "claimant": "alice"
            }]
        }))
        .unwrap_or_else(|error| panic!("task-complete batch HTTP request should decode: {error}"));
    assert_eq!(
        task_complete_batch.checkpoint_backend,
        QianjiBpmnWorkflowHttpCheckpointBackend::RuntimeValkey
    );
    assert_eq!(task_complete_batch.completions.len(), 1);
}

#[test]
fn bpmn_workflow_http_rejects_local_duckdb_backend_contract() {
    let error = match serde_json::from_value::<QianjiBpmnWorkflowHttpCheckpointBackend>(json!({
        "kind": "duckdb",
        "path": "state.duckdb",
    })) {
        Ok(backend) => {
            panic!("HTTP checkpoint backend should reject local DuckDB kind: {backend:?}")
        }
        Err(error) => error,
    };

    assert!(
        error.to_string().contains("unknown variant `duckdb`"),
        "unexpected decode error: {error}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn bpmn_workflow_http_snapshot_exposes_pending_human_task_contract() {
    let (form, assignment, lane, process) = http_snapshot_human_task_process();
    let package = Arc::new(BpmnPackage::new("pkg_http", vec![process]));
    let mut instance = create_instance(
        Arc::clone(&package),
        "review_flow",
        BpmnInstanceInit::new(
            "wf_http_human_task",
            json!({ "currentQuestion": "Ready?" }),
            10,
        ),
    )
    .unwrap_or_else(|error| panic!("instance should be created: {error:?}"));

    let outcome = advance_instance(
        package.as_ref(),
        &mut instance,
        &QianjiBpmnHostBridge::default(),
    )
    .await
    .unwrap_or_else(|error| panic!("instance should block on host work: {error:?}"));
    assert!(matches!(outcome, BpmnAdvanceOutcome::BlockedOnHost(_)));
    let token_id = instance.pending_host_work[0].token_id;
    claim_pending_human_task(
        &mut instance,
        PendingHumanTaskClaimRequest::from_input(PendingHumanTaskClaimInput {
            token_id: token_id.into(),
            process_id: "review_flow".into(),
            activity_id: "Task_Review".into(),
            claimant: "alice".to_string(),
            claimed_at_ms: 99.into(),
        }),
    )
    .unwrap_or_else(|error| panic!("human task claim should succeed: {error:?}"));

    let snapshot = QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&instance);

    assert_eq!(snapshot.pending_host_work_count, 1);
    assert_eq!(snapshot.pending_host_work.len(), 1);
    let work = &snapshot.pending_host_work[0];
    assert_eq!(work.token_id, instance.pending_host_work[0].token_id);
    assert_eq!(work.process_id.as_deref(), Some("review_flow"));
    assert_eq!(work.activity_id.as_deref(), Some("Task_Review"));
    assert_eq!(work.kind, PendingHostWorkKind::User);
    assert_eq!(work.form, Some(form));
    assert_eq!(work.assignment, Some(assignment));
    assert_eq!(work.lane, Some(lane));
    assert_eq!(
        work.claim,
        Some(PendingHostWorkClaim {
            claimant: "alice".to_string(),
            claimed_at_ms: 99,
        })
    );
    assert_human_task_event_kinds(
        &snapshot.human_task_events,
        &[
            BpmnHumanTaskLifecycleEventKind::Created,
            BpmnHumanTaskLifecycleEventKind::Claimed,
        ],
    );

    let snapshot_json = serde_json::to_value(&snapshot)
        .unwrap_or_else(|error| panic!("snapshot should serialize to JSON: {error}"));
    let work_json = &snapshot_json["pending_host_work"][0];
    assert_eq!(snapshot_json["instance_id"], json!("wf_http_human_task"));
    assert_eq!(work_json["token_id"], json!(token_id));
    assert_eq!(work_json["process_id"], json!("review_flow"));
    assert_eq!(work_json["activity_id"], json!("Task_Review"));
    assert_eq!(work_json["node_index"], json!(1));
    assert_eq!(work_json["kind"], json!("user"));
    assert_eq!(work_json["form"]["interaction_type"], json!("choice_input"));
    assert_eq!(work_json["form"]["question_ref"], json!("currentQuestion"));
    assert_eq!(work_json["form"]["result_output"], json!("answer"));
    assert_eq!(
        work_json["assignment"]["potential_owners"][0]["resource_ref"],
        json!("reviewers")
    );
    assert_eq!(work_json["lane"]["lane_set_id"], json!("LaneSet_Review"));
    assert_eq!(work_json["lane"]["lane_set_name"], json!("Ownership"));
    assert_eq!(work_json["lane"]["lane_id"], json!("Lane_Reviewer"));
    assert_eq!(work_json["lane"]["lane_name"], json!("Reviewer Lane"));
    assert_eq!(work_json["claim"]["claimant"], json!("alice"));
    assert_eq!(work_json["claim"]["claimed_at_ms"], json!(99));
    assert_eq!(
        snapshot_json["human_task_events"][0]["kind"],
        json!("created")
    );
    assert_eq!(
        snapshot_json["human_task_events"][1]["kind"],
        json!("claimed")
    );
    assert_eq!(
        snapshot_json["human_task_events"][1]["claimant"],
        json!("alice")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn bpmn_workflow_http_snapshot_exposes_host_dispatch_details() {
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_http", "service_flow", "digest_http_service"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "Task_Service", BpmnNodeKind::ServiceTask).with_task_io(
                BpmnTaskIoSpec::new()
                    .with_input(BpmnTaskInputBinding::new(
                        "amount",
                        BpmnTaskInputSource::variable("order.amount"),
                    ))
                    .with_input(BpmnTaskInputBinding::new(
                        "mode",
                        BpmnTaskInputSource::literal(r#"{"priority":"fast"}"#),
                    ))
                    .with_output(BpmnTaskOutputBinding::new("result", "service.result")),
            ),
            BpmnNodeSpec::new(2, "done", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    let package = Arc::new(BpmnPackage::new("pkg_http", vec![process]));
    let mut instance = create_instance(
        Arc::clone(&package),
        "service_flow",
        BpmnInstanceInit::new(
            "wf_http_service_dispatch",
            json!({ "order": { "amount": 7 } }),
            10,
        ),
    )
    .unwrap_or_else(|error| panic!("instance should be created: {error:?}"));

    let outcome = advance_instance(
        package.as_ref(),
        &mut instance,
        &QianjiBpmnHostBridge::default(),
    )
    .await
    .unwrap_or_else(|error| panic!("instance should block on service work: {error:?}"));
    assert!(matches!(outcome, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let snapshot = QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&instance);

    assert_eq!(snapshot.pending_host_work_count, 1);
    let work = &snapshot.pending_host_work[0];
    assert_eq!(work.kind, PendingHostWorkKind::Service);
    assert_eq!(work.node_id.as_deref(), Some("Task_Service"));
    assert_eq!(work.variables, json!({ "order": { "amount": 7 } }));
    assert_eq!(
        work.inputs,
        json!({ "amount": 7, "mode": { "priority": "fast" } })
    );
    assert_eq!(
        work.output_bindings,
        vec![BpmnTaskOutputBinding::new("result", "service.result")]
    );
    assert!(work.repeat.is_none());

    let snapshot_json = serde_json::to_value(&snapshot)
        .unwrap_or_else(|error| panic!("snapshot should serialize to JSON: {error}"));
    let work_json = &snapshot_json["pending_host_work"][0];
    assert_eq!(work_json["node_id"], json!("Task_Service"));
    assert_eq!(work_json["variables"]["order"]["amount"], json!(7));
    assert_eq!(work_json["inputs"]["amount"], json!(7));
    assert_eq!(work_json["inputs"]["mode"]["priority"], json!("fast"));
    assert_eq!(work_json["output_bindings"][0]["name"], json!("result"));
    assert_eq!(
        work_json["output_bindings"][0]["target_ref"],
        json!("service.result")
    );
}

#[tokio::test(flavor = "current_thread")]
#[cfg(feature = "valkey")]
async fn bpmn_workflow_http_start_at_routes_to_requested_host_task() {
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start for HTTP BPMN test: {error}"));
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_http_service_chain_bundle(&temp_dir);
    let router = workflow_http_router(valkey.url());
    let instance = unique_instance_id("wf_http_start_at_service");

    let started = post_json::<QianjiBpmnWorkflowRunHttpResponse>(
        router,
        "/workflows/start",
        json!({
            "bpmn_path": bpmn_path.display().to_string(),
            "process_id": "service_chain",
            "instance_id": instance.as_str(),
            "initial_variables": { "project": "qianji" },
            "start_at_node_id": "validate_contract"
        }),
    )
    .await;

    assert!(matches!(
        started.outcome,
        BpmnAdvanceOutcome::BlockedOnHost(_)
    ));
    assert!(!started.resumed_from_checkpoint);
    assert_eq!(started.workflow.pending_host_work_count, 1);
    let work = &started.workflow.pending_host_work[0];
    assert_eq!(work.kind, PendingHostWorkKind::Service);
    assert_eq!(work.node_id.as_deref(), Some("validate_contract"));
    assert_eq!(work.activity_id.as_deref(), Some("validate_contract"));
    assert_eq!(work.variables["project"], json!("qianji"));
}

#[tokio::test(flavor = "current_thread")]
#[cfg(feature = "valkey")]
async fn bpmn_workflow_http_batch_completion_completes_parallel_host_boundary() {
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start for HTTP BPMN test: {error}"));
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_http_parallel_service_bundle(&temp_dir);
    let router = workflow_http_router(valkey.url());
    let instance = unique_instance_id("wf_http_batch_parallel_service");

    let started = post_json::<QianjiBpmnWorkflowRunHttpResponse>(
        router.clone(),
        "/workflows/start",
        json!({
            "bpmn_path": bpmn_path.display().to_string(),
            "process_id": "parallel_batch_service",
            "instance_id": instance.as_str(),
            "initial_variables": {},
        }),
    )
    .await;

    assert!(matches!(
        started.outcome,
        BpmnAdvanceOutcome::BlockedOnHost(_)
    ));
    assert_eq!(started.workflow.pending_host_work_count, 2);
    let completions = started
        .workflow
        .pending_host_work
        .iter()
        .map(|work| {
            json!({
                "token_id": work.token_id,
                "process_id": "parallel_batch_service",
                "activity_id": work.activity_id.as_deref().unwrap_or("review"),
                "kind": "service",
                "data": {
                    "result": format!("completed_{}", work.token_id)
                }
            })
        })
        .collect::<Vec<_>>();

    let complete = post_json::<QianjiBpmnWorkflowRunHttpResponse>(
        router,
        format!("/workflows/{}/tasks/complete-batch", instance.as_str()).as_str(),
        json!({
            "bpmn_path": bpmn_path.display().to_string(),
            "completions": completions
        }),
    )
    .await;

    assert_eq!(complete.outcome, BpmnAdvanceOutcome::Completed);
    assert!(complete.resumed_from_checkpoint);
    assert_eq!(complete.workflow.pending_host_work_count, 0);
}

#[tokio::test(flavor = "current_thread")]
#[cfg(feature = "valkey")]
async fn bpmn_workflow_http_preserves_claim_identity_across_checkpoint_roundtrip() {
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start for HTTP BPMN test: {error}"));
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_http_user_task_bundle(&temp_dir);
    let router = workflow_http_router(valkey.url());
    let instance = unique_instance_id("wf_http_claim_identity_roundtrip");

    seed_http_runtime_valkey_user_task(valkey.url(), bpmn_path.as_path(), instance.as_str()).await;

    let initial_status = get_json::<QianjiBpmnWorkflowStatusHttpResponse>(
        router.clone(),
        format!("/workflows/{}", instance.as_str()).as_str(),
    )
    .await;
    let identity = HttpHumanTaskIdentity::from_status(instance.as_str(), &initial_status);
    assert_human_task_event_kinds(
        &initial_status.workflow.human_task_events,
        &[BpmnHumanTaskLifecycleEventKind::Created],
    );

    let claim = post_json::<QianjiBpmnWorkflowTaskClaimHttpResponse>(
        router.clone(),
        format!("/workflows/{}/tasks/claim", identity.instance).as_str(),
        json!({ "claim": identity.claim_payload("alice") }),
    )
    .await;
    identity.assert_claimed_work(&claim.claimed_work, "alice");
    assert_human_task_event_kinds(
        &claim.workflow.human_task_events,
        &[
            BpmnHumanTaskLifecycleEventKind::Created,
            BpmnHumanTaskLifecycleEventKind::Claimed,
        ],
    );

    let status = get_json::<QianjiBpmnWorkflowStatusHttpResponse>(
        router.clone(),
        format!("/workflows/{}", identity.instance).as_str(),
    )
    .await;
    identity.assert_claimed_work(&status.workflow.pending_host_work[0], "alice");
    assert_human_task_event_kinds(
        &status.workflow.human_task_events,
        &[
            BpmnHumanTaskLifecycleEventKind::Created,
            BpmnHumanTaskLifecycleEventKind::Claimed,
        ],
    );

    let release = post_json::<QianjiBpmnWorkflowTaskReleaseHttpResponse>(
        router.clone(),
        format!("/workflows/{}/tasks/release", identity.instance).as_str(),
        json!({ "release": identity.release_payload("alice") }),
    )
    .await;
    identity.assert_unclaimed_work(&release.released_work);
    assert_human_task_event_kinds(
        &release.workflow.human_task_events,
        &[
            BpmnHumanTaskLifecycleEventKind::Created,
            BpmnHumanTaskLifecycleEventKind::Claimed,
            BpmnHumanTaskLifecycleEventKind::Released,
        ],
    );

    post_json::<QianjiBpmnWorkflowTaskClaimHttpResponse>(
        router.clone(),
        format!("/workflows/{}/tasks/claim", identity.instance).as_str(),
        json!({ "claim": identity.claim_payload("alice") }),
    )
    .await;
    assert_wrong_claimant_http_completion_fails(
        router.clone(),
        &identity,
        bpmn_path.as_path(),
        "bob",
    )
    .await;

    let complete = post_json::<QianjiBpmnWorkflowRunHttpResponse>(
        router,
        format!("/workflows/{}/tasks/complete", identity.instance).as_str(),
        json!({
            "bpmn_path": bpmn_path.display().to_string(),
            "completion": identity.completion_payload("alice")
        }),
    )
    .await;
    assert_eq!(complete.outcome, BpmnAdvanceOutcome::Completed);
    assert!(complete.resumed_from_checkpoint);
    assert_eq!(complete.workflow.pending_host_work_count, 0);
    assert_human_task_event_kinds(
        &complete.workflow.human_task_events,
        &[
            BpmnHumanTaskLifecycleEventKind::Created,
            BpmnHumanTaskLifecycleEventKind::Claimed,
            BpmnHumanTaskLifecycleEventKind::Released,
            BpmnHumanTaskLifecycleEventKind::Claimed,
            BpmnHumanTaskLifecycleEventKind::Completed,
        ],
    );
    assert_eq!(
        complete.workflow.variables["answer"],
        json!("http_claim_identity")
    );
}

fn assert_human_task_event_kinds(
    events: &[xiuxian_qianji_bpmn_engine::BpmnHumanTaskLifecycleEvent],
    expected: &[BpmnHumanTaskLifecycleEventKind],
) {
    let actual = events
        .iter()
        .map(|event| event.kind.clone())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn http_snapshot_human_task_process() -> (
    BpmnHumanTaskFormSpec,
    BpmnHumanTaskAssignmentSpec,
    BpmnLaneMembershipSpec,
    BpmnProcessSpec,
) {
    let form = BpmnHumanTaskFormSpec::new("choice_input")
        .with_question_ref("currentQuestion")
        .with_choice(BpmnHumanTaskChoiceSpec::new("approve").with_label("Approve"))
        .with_result_output("answer");
    let assignment = BpmnHumanTaskAssignmentSpec::new().with_potential_owner(
        BpmnHumanTaskResourceRoleSpec::new()
            .with_name("review_team")
            .with_resource_ref("reviewers"),
    );
    let lane = BpmnLaneMembershipSpec::new()
        .with_lane_set_id("LaneSet_Review")
        .with_lane_set_name("Ownership")
        .with_lane_id("Lane_Reviewer")
        .with_lane_name("Reviewer Lane");
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_http", "review_flow", "digest_http"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "Task_Review", BpmnNodeKind::UserTask)
                .with_human_task_form(form.clone())
                .with_human_task_assignment(assignment.clone())
                .with_lane(lane.clone()),
            BpmnNodeSpec::new(2, "done", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    (form, assignment, lane, process)
}

#[cfg(feature = "valkey")]
struct HttpHumanTaskIdentity {
    instance: String,
    token: u64,
    activity: String,
}

#[cfg(feature = "valkey")]
impl HttpHumanTaskIdentity {
    const PROCESS: &'static str = "review";

    fn from_status(instance: &str, status: &QianjiBpmnWorkflowStatusHttpResponse) -> Self {
        assert_eq!(status.workflow.instance_id, instance);
        assert_eq!(status.workflow.pending_host_work_count, 1);
        let work = &status.workflow.pending_host_work[0];
        let identity = Self {
            instance: instance.to_string(),
            token: work.token_id,
            activity: work
                .activity_id
                .clone()
                .unwrap_or_else(|| panic!("HTTP pending work should carry activity identity"))
                .to_string(),
        };
        identity.assert_unclaimed_work(work);
        identity
    }

    fn assert_unclaimed_work(&self, work: &crate::QianjiBpmnPendingHostWorkHttpResponse) {
        self.assert_identity(work);
        assert!(work.claim.is_none());
    }

    fn assert_claimed_work(
        &self,
        work: &crate::QianjiBpmnPendingHostWorkHttpResponse,
        claimant: &str,
    ) {
        self.assert_identity(work);
        assert_eq!(
            work.claim.as_ref().map(|claim| claim.claimant.as_str()),
            Some(claimant)
        );
    }

    fn assert_identity(&self, work: &crate::QianjiBpmnPendingHostWorkHttpResponse) {
        assert_eq!(work.token_id, self.token);
        assert_eq!(work.process_id.as_deref(), Some(Self::PROCESS));
        assert_eq!(work.activity_id.as_deref(), Some(self.activity.as_str()));
        assert_eq!(work.kind, PendingHostWorkKind::User);
    }

    fn claim_payload(&self, claimant: &str) -> serde_json::Value {
        json!({
            "token_id": self.token,
            "process_id": Self::PROCESS,
            "activity_id": self.activity,
            "claimant": claimant
        })
    }

    fn release_payload(&self, claimant: &str) -> serde_json::Value {
        self.claim_payload(claimant)
    }

    fn completion_payload(&self, claimant: &str) -> serde_json::Value {
        json!({
            "token_id": self.token,
            "process_id": Self::PROCESS,
            "activity_id": self.activity,
            "kind": "user",
            "data": {
                "answer": "http_claim_identity"
            },
            "claimant": claimant
        })
    }
}

#[cfg(feature = "valkey")]
async fn assert_wrong_claimant_http_completion_fails(
    router: Router,
    identity: &HttpHumanTaskIdentity,
    bpmn_path: &Path,
    claimant: &str,
) {
    let (status, body) = request_json(
        router,
        Method::POST,
        format!("/workflows/{}/tasks/complete", identity.instance).as_str(),
        Some(json!({
            "bpmn_path": bpmn_path.display().to_string(),
            "completion": identity.completion_payload(claimant)
        })),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error = serde_json::from_value::<QianjiBpmnWorkflowHttpErrorBody>(body)
        .unwrap_or_else(|error| panic!("HTTP error body should decode: {error}"));
    assert_eq!(error.code, "workflow_orchestration_failed");
    assert!(
        error
            .message
            .contains("expected claimant 'alice', got 'bob'")
    );
}

#[cfg(feature = "valkey")]
fn workflow_http_router(valkey_url: &str) -> Router {
    qianji_bpmn_workflow_router(QianjiBpmnWorkflowHttpState::new(
        workflow_control_service(valkey_url),
        QianjiBpmnHostBridge::default(),
    ))
}

#[cfg(feature = "valkey")]
fn workflow_control_service(valkey_url: &str) -> QianjiBpmnWorkflowControlService {
    let runtime_env = QianjiRuntimeEnv {
        qianji_checkpoint_valkey_url: Some(valkey_url.to_string()),
        ..QianjiRuntimeEnv::default()
    };
    QianjiBpmnWorkflowControlService::new()
        .with_runtime_env(runtime_env)
        .with_scheduler_identity(SchedulerAgentIdentity::new(
            Some("http-worker".to_string()),
            Some("http-manager".to_string()),
        ))
}

#[cfg(feature = "valkey")]
async fn seed_http_runtime_valkey_user_task(valkey_url: &str, bpmn_path: &Path, instance: &str) {
    let service = workflow_control_service(valkey_url);
    let request = QianjiBpmnWorkflowStartRequest {
        bpmn_path: bpmn_path.to_path_buf(),
        dmn_paths: Vec::new(),
        process_id: HttpHumanTaskIdentity::PROCESS.to_string().into(),
        instance_id: instance.to_string().into(),
        initial_variables: Some(json!({ "risk": "high" })),
        start_at_node_id: None,
        checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey),
    };
    let prepared = service
        .prepare_start_workflow(&request)
        .unwrap_or_else(|error| panic!("HTTP seed workflow should prepare: {error}"));
    let report = service
        .start_prepared_workflow_until_human_boundary(
            prepared,
            &QianjiBpmnHostBridge::default(),
            false,
            |_, _| {},
        )
        .await
        .unwrap_or_else(|error| panic!("HTTP seed workflow should reach human boundary: {error}"));

    assert!(matches!(
        report.execution.outcome,
        BpmnAdvanceOutcome::BlockedOnHost(_)
    ));
    assert!(report.execution.checkpoint_saved);
}

#[cfg(feature = "valkey")]
async fn post_json<T>(router: Router, uri: &str, payload: serde_json::Value) -> T
where
    T: DeserializeOwned,
{
    let (status, body) = request_json(router, Method::POST, uri, Some(payload)).await;
    assert_eq!(status, StatusCode::OK, "unexpected HTTP body: {body}");
    serde_json::from_value(body)
        .unwrap_or_else(|error| panic!("HTTP response body should decode: {error}"))
}

#[cfg(feature = "valkey")]
async fn get_json<T>(router: Router, uri: &str) -> T
where
    T: DeserializeOwned,
{
    let (status, body) = request_json(router, Method::GET, uri, None).await;
    assert_eq!(status, StatusCode::OK, "unexpected HTTP body: {body}");
    serde_json::from_value(body)
        .unwrap_or_else(|error| panic!("HTTP response body should decode: {error}"))
}

#[cfg(feature = "valkey")]
async fn request_json(
    router: Router,
    method: Method,
    uri: &str,
    payload: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let body = payload.map_or_else(Body::empty, |value| Body::from(value.to_string()));
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .unwrap_or_else(|error| panic!("HTTP request should build: {error}"));
    let response = router
        .oneshot(request)
        .await
        .unwrap_or_else(|error| panic!("HTTP router should answer request: {error}"));
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("HTTP body should buffer: {error}"));
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("HTTP body should be JSON: {error}"));
    (status, body)
}

#[cfg(feature = "valkey")]
fn write_http_user_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("http-user-task.bpmn");
    fs::write(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_http_review">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="review_task">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="review_task_output_answer" name="answer" />
        <bpmn:outputSet id="review_task_output_set">
          <bpmn:dataOutputRefs>review_task_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>review_task_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>answer</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:userTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review_task" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review_task" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    )
    .unwrap_or_else(|error| panic!("HTTP BPMN fixture should write: {error}"));
    bpmn_path
}

#[cfg(feature = "valkey")]
fn write_http_service_chain_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("http-service-chain.bpmn");
    fs::write(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_http_service_chain">
  <bpmn:process id="service_chain" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="resolve_project" />
    <bpmn:serviceTask id="validate_contract" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="resolve_project" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="resolve_project" targetRef="validate_contract" />
    <bpmn:sequenceFlow id="flow_3" sourceRef="validate_contract" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    )
    .unwrap_or_else(|error| panic!("HTTP BPMN fixture should write: {error}"));
    bpmn_path
}

#[cfg(feature = "valkey")]
fn write_http_parallel_service_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("http-parallel-service.bpmn");
    fs::write(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_http_parallel_service">
  <bpmn:process id="parallel_batch_service" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="review">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="review_output_result" name="result" />
        <bpmn:outputSet id="review_output_set">
          <bpmn:dataOutputRefs>review_output_result</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:multiInstanceLoopCharacteristics>
        <bpmn:loopCardinality>2</bpmn:loopCardinality>
      </bpmn:multiInstanceLoopCharacteristics>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>review_output_result</bpmn:sourceRef>
        <bpmn:targetRef>results</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_start_review" sourceRef="start" targetRef="review" />
    <bpmn:sequenceFlow id="flow_review_end" sourceRef="review" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    )
    .unwrap_or_else(|error| panic!("HTTP parallel BPMN fixture should write: {error}"));
    bpmn_path
}
