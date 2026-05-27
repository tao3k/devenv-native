use crate::qianji_test_valkey_support::TestValkey;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use tower::util::ServiceExt;
use xiuxian_qianji_control::{
    ActivityStatus, ControlEventKind, ControlLedger, DuckDbControlLedger, InMemoryHotStateStore,
    RunId,
};

use super::support::{must_ok, write_file};
use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::{build_qianji_server_router, build_workflow_control_service};
use crate::{QianjiBpmnHostBridge, QianjiBpmnWorkflowHttpState, qianji_bpmn_workflow_router};

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_completion_records_host_work_activity_evidence() {
    let proof =
        start_mapped_service_evidence_workflow("qianji_server_host_work_activity_evidence").await;

    let complete_payload = json!({
        "bpmn_path": proof.bpmn_path.display().to_string(),
        "completion": {
            "token_id": proof.service_token,
            "process_id": "mapped_service_boundary",
            "activity_id": "resolve_project",
            "kind": "service",
            "data": {
                "resolvedProject": true
            }
        }
    });
    let complete_response = proof
        .router
        .oneshot(post_json(
            format!("/workflows/{}/tasks/complete", proof.instance_id).as_str(),
            &complete_payload,
        ))
        .await
        .unwrap_or_else(|error| panic!("service completion route should respond: {error}"));
    assert_eq!(complete_response.status(), StatusCode::OK);

    let (event_kinds, activity_status) =
        replay_activity_evidence(&proof.ledger_path, proof.instance_id);
    assert_eq!(
        event_kinds,
        vec![
            "run_created",
            "activity_scheduled",
            "activity_started",
            "activity_completed",
        ]
    );
    assert_eq!(activity_status, ActivityStatus::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_failure_records_host_work_activity_evidence_without_advancing() {
    let proof =
        start_mapped_service_evidence_workflow("qianji_server_host_work_failure_evidence").await;
    let fail_payload = service_failure_payload(&proof);

    assert_failure_route_preserves_checkpoint(&proof, &fail_payload).await;
    assert_failed_control_history(&proof).await;
    assert_failed_control_summary(&proof).await;
    assert_failed_control_recovery(&proof).await;
    assert_failed_control_diagnostics(&proof).await;
    assert_failed_activity_evidence(&proof);
    assert_workflow_still_blocked(&proof).await;
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_recovery_apply_records_bounded_attempt() {
    let proof = start_mapped_service_evidence_workflow_with_recovery_hot_state(
        "qianji_server_host_work_recovery_apply",
    )
    .await;
    let fail_payload = service_failure_payload(&proof);

    assert_failure_route_preserves_checkpoint(&proof, &fail_payload).await;

    let apply_response = proof
        .router
        .clone()
        .oneshot(post_json(
            format!(
                "/control/runs/bpmn.workflow.{}/recovery/apply",
                proof.instance_id
            )
            .as_str(),
            &json!({
                "occurred_at_ms": 99_000,
                "attempt": 1,
                "reason": "operator requested bounded recovery",
                "max_attempts": 1
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("control recovery apply route should respond: {error}"));
    let status = apply_response.status();
    let body = response_json(apply_response).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "control recovery apply route should succeed: {body}"
    );
    assert_eq!(
        body["run_id"],
        format!("bpmn.workflow.{}", proof.instance_id)
    );
    assert_eq!(
        body["application"]["action_results"][0]["result"]["status"],
        "not_applicable"
    );
    assert_eq!(
        body["application"]["action_results"][0]["result"]["reason"],
        "unsupported_action"
    );
    assert_eq!(body["diagnostics"]["summary"]["event_count"], 5);
    assert_eq!(
        body["diagnostics"]["recovery"]["summary"],
        body["diagnostics"]["summary"]["recovery"]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_recovery_apply_requires_hot_state() {
    let proof = start_mapped_service_evidence_workflow_with_control_ledger_only(
        "qianji_server_host_work_recovery_apply_no_hot_state",
    )
    .await;
    let fail_payload = service_failure_payload(&proof);

    assert_failure_route_preserves_checkpoint(&proof, &fail_payload).await;

    let apply_response = proof
        .router
        .oneshot(post_json(
            format!(
                "/control/runs/bpmn.workflow.{}/recovery/apply",
                proof.instance_id
            )
            .as_str(),
            &json!({
                "occurred_at_ms": 99_000,
                "attempt": 1,
                "reason": "operator requested bounded recovery",
                "max_attempts": 1
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("control recovery apply route should respond: {error}"));
    let status = apply_response.status();
    let body = response_json(apply_response).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "control_hot_state_unavailable");
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_recovery_apply_rejects_invalid_policy() {
    let proof = start_mapped_service_evidence_workflow_with_recovery_hot_state(
        "qianji_server_host_work_recovery_apply_invalid_policy",
    )
    .await;
    let uri = format!(
        "/control/runs/bpmn.workflow.{}/recovery/apply",
        proof.instance_id
    );

    let blank_reason_response = proof
        .router
        .clone()
        .oneshot(post_json(
            uri.as_str(),
            &json!({
                "occurred_at_ms": 99_000,
                "attempt": 1,
                "reason": " ",
                "max_attempts": 1
            }),
        ))
        .await
        .unwrap_or_else(|error| {
            panic!("blank reason recovery apply route should respond: {error}")
        });
    let blank_reason_status = blank_reason_response.status();
    let blank_reason_body = response_json(blank_reason_response).await;

    assert_eq!(blank_reason_status, StatusCode::BAD_REQUEST);
    assert_eq!(blank_reason_body["code"], "invalid_recovery_reason");

    let zero_attempts_response = proof
        .router
        .oneshot(post_json(
            uri.as_str(),
            &json!({
                "occurred_at_ms": 99_000,
                "attempt": 1,
                "reason": "operator requested bounded recovery",
                "max_attempts": 0
            }),
        ))
        .await
        .unwrap_or_else(|error| {
            panic!("zero max attempts recovery apply route should respond: {error}")
        });
    let zero_attempts_status = zero_attempts_response.status();
    let zero_attempts_body = response_json(zero_attempts_response).await;

    assert_eq!(zero_attempts_status, StatusCode::BAD_REQUEST);
    assert_eq!(zero_attempts_body["code"], "invalid_recovery_policy");
}

fn service_failure_payload(proof: &ActivityEvidenceProof) -> Value {
    json!({
        "bpmn_path": proof.bpmn_path.display().to_string(),
        "failure": {
            "token_id": proof.service_token,
            "process_id": "mapped_service_boundary",
            "activity_id": "resolve_project",
            "kind": "service",
            "error_code": "native_host_execution_failed",
            "message": "agent execution failed",
            "retryable": true,
            "metadata": {
                "source": "pi-wendao"
            }
        }
    })
}

async fn assert_failure_route_preserves_checkpoint(
    proof: &ActivityEvidenceProof,
    fail_payload: &Value,
) {
    let fail_response = proof
        .router
        .clone()
        .oneshot(post_json(
            format!("/workflows/{}/tasks/fail", proof.instance_id).as_str(),
            fail_payload,
        ))
        .await
        .unwrap_or_else(|error| panic!("task failure route should respond: {error}"));
    let fail_status = fail_response.status();
    let fail_body = response_json(fail_response).await;
    assert_eq!(
        fail_status,
        StatusCode::OK,
        "task failure route should preserve checkpoint status: {fail_body}"
    );
    assert_eq!(fail_body["workflow"]["pending_host_work_count"], 1);
    assert_eq!(
        fail_body["workflow"]["pending_host_work"][0]["activity_id"],
        "resolve_project"
    );
}

async fn assert_failed_control_history(proof: &ActivityEvidenceProof) {
    let history_response = proof
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/bpmn.workflow.{}/history",
            proof.instance_id
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("control history route should respond: {error}"));
    let history_body = response_json(history_response).await;
    assert_eq!(
        history_body["run_id"],
        format!("bpmn.workflow.{}", proof.instance_id)
    );
    assert_eq!(history_body["event_count"], 4);
    assert_eq!(
        control_history_event_kinds(&history_body),
        vec![
            "run_created",
            "activity_scheduled",
            "activity_started",
            "activity_failed",
        ]
    );
    assert_eq!(
        history_body["events"][3]["event"]["kind"]["failure"]["message"],
        "agent execution failed"
    );
}

async fn assert_failed_control_summary(proof: &ActivityEvidenceProof) {
    let summary_response = proof
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/bpmn.workflow.{}/summary",
            proof.instance_id
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("control summary route should respond: {error}"));
    let summary_body = response_json(summary_response).await;
    assert_eq!(
        summary_body["run_id"],
        format!("bpmn.workflow.{}", proof.instance_id)
    );
    assert_eq!(summary_body["summary"]["event_count"], 4);
    assert_eq!(summary_body["summary"]["activities"]["total"], 1);
    assert_eq!(summary_body["summary"]["activities"]["failed"], 1);
    assert_eq!(summary_body["summary"]["activities"]["in_flight"], 0);
}

async fn assert_failed_control_recovery(proof: &ActivityEvidenceProof) {
    let recovery_response = proof
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/bpmn.workflow.{}/recovery",
            proof.instance_id
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("control recovery route should respond: {error}"));
    let recovery_body = response_json(recovery_response).await;
    assert_eq!(
        recovery_body["run_id"],
        format!("bpmn.workflow.{}", proof.instance_id)
    );
    assert_eq!(recovery_body["recovery"]["summary"]["total_actions"], 1);
    assert_eq!(
        recovery_body["recovery"]["summary"]["review_retryable_activities"],
        1
    );
    assert_eq!(
        recovery_body["recovery"]["plan"]["actions"][0]["action"],
        "review_retryable_activity"
    );
}

async fn assert_failed_control_diagnostics(proof: &ActivityEvidenceProof) {
    let diagnostics_response = proof
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/bpmn.workflow.{}/diagnostics",
            proof.instance_id
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("control diagnostics route should respond: {error}"));
    let diagnostics_body = response_json(diagnostics_response).await;
    assert_eq!(
        diagnostics_body["run_id"],
        format!("bpmn.workflow.{}", proof.instance_id)
    );
    assert_eq!(diagnostics_body["diagnostics"]["summary"]["event_count"], 4);
    assert_eq!(
        diagnostics_body["diagnostics"]["recovery"]["summary"],
        diagnostics_body["diagnostics"]["summary"]["recovery"]
    );
    assert_eq!(
        diagnostics_body["diagnostics"]["recovery"]["plan"]["actions"][0]["action"],
        "review_retryable_activity"
    );
}

fn assert_failed_activity_evidence(proof: &ActivityEvidenceProof) {
    let (event_kinds, activity_status) =
        replay_activity_evidence(&proof.ledger_path, proof.instance_id);
    assert_eq!(
        event_kinds,
        vec![
            "run_created",
            "activity_scheduled",
            "activity_started",
            "activity_failed",
        ]
    );
    assert_eq!(activity_status, ActivityStatus::Failed);
}

async fn assert_workflow_still_blocked(proof: &ActivityEvidenceProof) {
    let status_response = proof
        .router
        .clone()
        .oneshot(get(format!("/workflows/{}", proof.instance_id).as_str()))
        .await
        .unwrap_or_else(|error| panic!("workflow status route should respond: {error}"));
    let status_body = response_json(status_response).await;
    assert_eq!(status_body["workflow"]["pending_host_work_count"], 1);
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_control_history_requires_configured_ledger() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let router = server_router_without_control_ledger(temp_dir.path().join("unused-flowhub"));

    let response = router
        .clone()
        .oneshot(get("/control/runs/bpmn.workflow.missing/history"))
        .await
        .unwrap_or_else(|error| panic!("control history route should respond: {error}"));
    let status = response.status();
    let body = response_json(response).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "control_ledger_unavailable");

    let response = router
        .clone()
        .oneshot(get("/control/runs/bpmn.workflow.missing/summary"))
        .await
        .unwrap_or_else(|error| panic!("control summary route should respond: {error}"));
    let status = response.status();
    let body = response_json(response).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "control_ledger_unavailable");

    let response = router
        .clone()
        .oneshot(get("/control/runs/bpmn.workflow.missing/recovery"))
        .await
        .unwrap_or_else(|error| panic!("control recovery route should respond: {error}"));
    let status = response.status();
    let body = response_json(response).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "control_ledger_unavailable");

    let response = router
        .clone()
        .oneshot(get("/control/runs/bpmn.workflow.missing/diagnostics"))
        .await
        .unwrap_or_else(|error| panic!("control diagnostics route should respond: {error}"));
    let status = response.status();
    let body = response_json(response).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "control_ledger_unavailable");

    let response = router
        .oneshot(post_json(
            "/control/runs/bpmn.workflow.missing/recovery/apply",
            &json!({
                "occurred_at_ms": 1,
                "attempt": 1,
                "reason": "operator requested bounded recovery",
                "max_attempts": 1
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("control recovery apply route should respond: {error}"));
    let status = response.status();
    let body = response_json(response).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "control_ledger_unavailable");
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_failure_rejects_blank_message_without_partial_activity_evidence() {
    let proof =
        start_mapped_service_evidence_workflow("qianji_server_host_work_blank_failure_evidence")
            .await;

    let fail_payload = json!({
        "bpmn_path": proof.bpmn_path.display().to_string(),
        "failure": {
            "token_id": proof.service_token,
            "process_id": "mapped_service_boundary",
            "activity_id": "resolve_project",
            "kind": "service",
            "error_code": "native_host_execution_failed",
            "message": " ",
            "retryable": true,
            "metadata": {
                "source": "pi-wendao"
            }
        }
    });
    let fail_response = proof
        .router
        .oneshot(post_json(
            format!("/workflows/{}/tasks/fail", proof.instance_id).as_str(),
            &fail_payload,
        ))
        .await
        .unwrap_or_else(|error| panic!("blank task failure route should respond: {error}"));
    assert_eq!(fail_response.status(), StatusCode::BAD_REQUEST);
    assert!(
        replay_activity_evidence_event_kinds(&proof.ledger_path, proof.instance_id).is_empty(),
        "blank failure payload must not leave partial ActivityTask evidence"
    );
}

struct ActivityEvidenceProof {
    _temp_dir: TempDir,
    _valkey: TestValkey,
    router: Router,
    bpmn_path: PathBuf,
    ledger_path: PathBuf,
    instance_id: &'static str,
    service_token: u64,
}

async fn start_mapped_service_evidence_workflow(
    instance_id: &'static str,
) -> ActivityEvidenceProof {
    start_mapped_service_evidence_workflow_with_router(
        instance_id,
        server_router_with_valkey_and_control_ledger,
    )
    .await
}

async fn start_mapped_service_evidence_workflow_with_recovery_hot_state(
    instance_id: &'static str,
) -> ActivityEvidenceProof {
    start_mapped_service_evidence_workflow_with_router(
        instance_id,
        server_router_with_valkey_control_ledger_and_in_memory_hot_state,
    )
    .await
}

async fn start_mapped_service_evidence_workflow_with_control_ledger_only(
    instance_id: &'static str,
) -> ActivityEvidenceProof {
    start_mapped_service_evidence_workflow_with_router(
        instance_id,
        server_router_with_valkey_control_ledger_only,
    )
    .await
}

async fn start_mapped_service_evidence_workflow_with_router(
    instance_id: &'static str,
    router_builder: fn(PathBuf, String, PathBuf) -> Router,
) -> ActivityEvidenceProof {
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start for qianji-server proof: {error}"));
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_mapped_service_boundary_bpmn(temp_dir.path());
    let ledger_path = temp_dir.path().join("qianji-control.duckdb");
    let router = router_builder(
        temp_dir.path().join("unused-flowhub"),
        valkey.url().to_string(),
        ledger_path.clone(),
    );

    let start_payload = json!({
        "bpmn_path": bpmn_path.display().to_string(),
        "process_id": "mapped_service_boundary",
        "instance_id": instance_id,
        "initial_variables": {}
    });
    let start_response = router
        .clone()
        .oneshot(post_json("/workflows/start", &start_payload))
        .await
        .unwrap_or_else(|error| panic!("workflow start route should respond: {error}"));
    let start_body = response_json(start_response).await;
    let service_token = start_body["workflow"]["pending_host_work"][0]["token_id"]
        .as_u64()
        .unwrap_or_else(|| panic!("pending service token should be an integer"));

    ActivityEvidenceProof {
        _temp_dir: temp_dir,
        _valkey: valkey,
        router,
        bpmn_path,
        ledger_path,
        instance_id,
        service_token,
    }
}

fn replay_activity_evidence(
    ledger_path: &Path,
    instance_id: &str,
) -> (Vec<&'static str>, ActivityStatus) {
    let event_kinds = replay_activity_evidence_event_kinds(ledger_path, instance_id);
    let ledger = DuckDbControlLedger::open(ledger_path)
        .unwrap_or_else(|error| panic!("control ledger should reopen: {error}"));
    let run_id = RunId::new(format!("bpmn.workflow.{instance_id}"))
        .unwrap_or_else(|error| panic!("run id should build: {error}"));
    let view = ledger
        .load_run_view(&run_id)
        .unwrap_or_else(|error| panic!("activity evidence run should replay: {error}"));
    let activity = view
        .activities
        .values()
        .next()
        .unwrap_or_else(|| panic!("activity evidence should include one activity"));
    assert_eq!(
        activity
            .task
            .as_ref()
            .unwrap_or_else(|| panic!("activity task should be retained"))
            .activity_type
            .as_str(),
        "bpmn.host_work"
    );
    (event_kinds, activity.status)
}

fn replay_activity_evidence_event_kinds(
    ledger_path: &Path,
    instance_id: &str,
) -> Vec<&'static str> {
    let ledger = DuckDbControlLedger::open(ledger_path)
        .unwrap_or_else(|error| panic!("control ledger should reopen: {error}"));
    let run_id = RunId::new(format!("bpmn.workflow.{instance_id}"))
        .unwrap_or_else(|error| panic!("run id should build: {error}"));
    let records = ledger
        .load_events(&run_id)
        .unwrap_or_else(|error| panic!("activity evidence events should load: {error}"));
    records
        .iter()
        .map(|record| match &record.event.kind {
            ControlEventKind::RunCreated { .. } => "run_created",
            ControlEventKind::ActivityScheduled { .. } => "activity_scheduled",
            ControlEventKind::ActivityStarted { .. } => "activity_started",
            ControlEventKind::ActivityCompleted { .. } => "activity_completed",
            ControlEventKind::ActivityFailed { .. } => "activity_failed",
            other => panic!("unexpected activity evidence event: {other:?}"),
        })
        .collect::<Vec<_>>()
}

fn control_history_event_kinds(history_body: &Value) -> Vec<&str> {
    history_body["events"]
        .as_array()
        .unwrap_or_else(|| panic!("control history should include events: {history_body}"))
        .iter()
        .map(|record| {
            record["event"]["kind"]["event"]
                .as_str()
                .unwrap_or_else(|| panic!("control event kind should be tagged: {record}"))
        })
        .collect()
}

fn server_router_without_control_ledger(flowhub_root: PathBuf) -> Router {
    let command = QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some("redis://127.0.0.1:0".to_string()),
        require_valkey_ready: None,
        flowhub_root: Some(flowhub_root),
        control_ledger_path: None,
    };
    must_ok(
        build_qianji_server_router(&command),
        "qianji-server router should build without a control ledger",
    )
}

fn server_router_with_valkey_and_control_ledger(
    flowhub_root: PathBuf,
    valkey_url: String,
    control_ledger_path: PathBuf,
) -> Router {
    let command = QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some(valkey_url),
        require_valkey_ready: None,
        flowhub_root: Some(flowhub_root),
        control_ledger_path: Some(control_ledger_path),
    };
    must_ok(
        build_qianji_server_router(&command),
        "qianji-server router should build with explicit control ledger",
    )
}

fn server_router_with_valkey_control_ledger_and_in_memory_hot_state(
    flowhub_root: PathBuf,
    valkey_url: String,
    control_ledger_path: PathBuf,
) -> Router {
    let command = QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some(valkey_url),
        require_valkey_ready: None,
        flowhub_root: Some(flowhub_root),
        control_ledger_path: Some(control_ledger_path.clone()),
    };
    let ledger = must_ok(
        DuckDbControlLedger::open(&control_ledger_path),
        "control ledger should open for recovery apply test",
    );
    qianji_bpmn_workflow_router(
        QianjiBpmnWorkflowHttpState::new(
            build_workflow_control_service(&command),
            QianjiBpmnHostBridge::default(),
        )
        .with_activity_evidence_ledger(Arc::new(ledger))
        .with_recovery_hot_state(Arc::new(InMemoryHotStateStore::new())),
    )
}

fn server_router_with_valkey_control_ledger_only(
    flowhub_root: PathBuf,
    valkey_url: String,
    control_ledger_path: PathBuf,
) -> Router {
    let command = QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some(valkey_url),
        require_valkey_ready: None,
        flowhub_root: Some(flowhub_root),
        control_ledger_path: Some(control_ledger_path.clone()),
    };
    let ledger = must_ok(
        DuckDbControlLedger::open(&control_ledger_path),
        "control ledger should open for recovery apply hot-state guard test",
    );
    qianji_bpmn_workflow_router(
        QianjiBpmnWorkflowHttpState::new(
            build_workflow_control_service(&command),
            QianjiBpmnHostBridge::default(),
        )
        .with_activity_evidence_ledger(Arc::new(ledger)),
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("GET request should build: {error}"))
}

fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap_or_else(|error| panic!("POST request should build: {error}"))
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("response body should decode as JSON: {error}"))
}

fn write_mapped_service_boundary_bpmn(root: &Path) -> PathBuf {
    let bpmn_path = root.join("mapped-service-boundary.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_mapped_service_boundary">
  <bpmn:process id="mapped_service_boundary" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="resolve_project">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="resolve_project_output_resolvedProject" name="resolvedProject" />
        <bpmn:outputSet id="resolve_project_output_set">
          <bpmn:dataOutputRefs>resolve_project_output_resolvedProject</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>resolve_project_output_resolvedProject</bpmn:sourceRef>
        <bpmn:targetRef>resolvedProject</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="validate_contract" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_resolve" sourceRef="start" targetRef="resolve_project" />
    <bpmn:sequenceFlow id="flow_resolve_validate" sourceRef="resolve_project" targetRef="validate_contract" />
    <bpmn:sequenceFlow id="flow_validate_done" sourceRef="validate_contract" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}
