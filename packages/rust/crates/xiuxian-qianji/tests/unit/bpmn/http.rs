use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowActionHttpRequest,
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowSnapshotHttpResponse,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskCompleteHttpRequest, QianjiBpmnWorkflowTaskReleaseHttpRequest,
};
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnHumanTaskAssignmentSpec, BpmnHumanTaskChoiceSpec,
    BpmnHumanTaskFormSpec, BpmnHumanTaskResourceRoleSpec, BpmnInstanceInit, BpmnNodeKind,
    BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, PendingHostWorkClaim, PendingHostWorkKind,
    PendingHumanTaskClaimRequest, ProcessKey, advance_instance, claim_pending_human_task,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

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
    let form = BpmnHumanTaskFormSpec::new("choice_input")
        .with_question_ref("currentQuestion")
        .with_choice(BpmnHumanTaskChoiceSpec::new("approve").with_label("Approve"))
        .with_result_output("answer");
    let assignment = BpmnHumanTaskAssignmentSpec::new().with_potential_owner(
        BpmnHumanTaskResourceRoleSpec::new()
            .with_name("review_team")
            .with_resource_ref("reviewers"),
    );
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_http", "review_flow", "digest_http"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "Task_Review", BpmnNodeKind::UserTask)
                .with_human_task_form(form.clone())
                .with_human_task_assignment(assignment.clone()),
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
        PendingHumanTaskClaimRequest::new(token_id, "review_flow", "Task_Review", "alice", 99),
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
    assert_eq!(
        work.claim,
        Some(PendingHostWorkClaim {
            claimant: "alice".to_string(),
            claimed_at_ms: 99,
        })
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
    assert_eq!(work_json["claim"]["claimant"], json!("alice"));
    assert_eq!(work_json["claim"]["claimed_at_ms"], json!(99));
}
