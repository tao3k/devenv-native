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

use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::{build_qianji_server_router, build_workflow_control_service};
use crate::qianji_server_cli::tests::support::{must_ok, write_file};
use crate::{QianjiBpmnHostBridge, QianjiBpmnWorkflowHttpState, qianji_bpmn_workflow_router};

pub(super) fn service_failure_payload(proof: &ActivityEvidenceProof) -> Value {
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

pub(super) async fn assert_failure_route_preserves_checkpoint(
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

pub(super) async fn assert_failed_control_history(proof: &ActivityEvidenceProof) {
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
    assert!(
        history_body["event_count"].as_u64().unwrap_or_default() >= 4,
        "control history should include BPMN trace plus activity failure evidence: {history_body}"
    );
    let event_kinds = control_history_event_kinds(&history_body);
    assert!(event_kinds.contains(&"run_created"));
    assert!(event_kinds.contains(&"step_created"));
    assert!(event_kinds.contains(&"tool_call_recorded"));
    assert!(event_kinds.contains(&"activity_scheduled"));
    assert!(event_kinds.contains(&"activity_started"));
    assert!(event_kinds.contains(&"activity_failed"));
    let failed_event = history_body["events"]
        .as_array()
        .unwrap_or_else(|| panic!("control history should include events: {history_body}"))
        .iter()
        .find(|record| record["event"]["kind"]["event"] == "activity_failed")
        .unwrap_or_else(|| {
            panic!("control history should include activity_failed: {history_body}")
        });
    assert_eq!(
        failed_event["event"]["kind"]["failure"]["message"],
        "agent execution failed"
    );
}

pub(super) async fn assert_failed_control_summary(proof: &ActivityEvidenceProof) {
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
    assert!(
        summary_body["summary"]["event_count"]
            .as_u64()
            .unwrap_or_default()
            >= 4
    );
    assert_eq!(summary_body["summary"]["activities"]["total"], 1);
    assert_eq!(summary_body["summary"]["activities"]["failed"], 1);
    assert_eq!(summary_body["summary"]["activities"]["in_flight"], 0);
}

pub(super) async fn assert_failed_control_recovery(proof: &ActivityEvidenceProof) {
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

pub(super) async fn assert_failed_control_diagnostics(proof: &ActivityEvidenceProof) {
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
    assert!(
        diagnostics_body["diagnostics"]["summary"]["event_count"]
            .as_u64()
            .unwrap_or_default()
            >= 4
    );
    assert_eq!(
        diagnostics_body["diagnostics"]["recovery"]["summary"],
        diagnostics_body["diagnostics"]["summary"]["recovery"]
    );
    assert_eq!(
        diagnostics_body["diagnostics"]["recovery"]["plan"]["actions"][0]["action"],
        "review_retryable_activity"
    );
}

pub(super) fn assert_failed_activity_evidence(proof: &ActivityEvidenceProof) {
    let (event_kinds, activity_status) =
        replay_activity_evidence(&proof.ledger_path, proof.instance_id);
    assert_eq!(
        event_kinds,
        vec!["activity_scheduled", "activity_started", "activity_failed",]
    );
    assert_eq!(activity_status, ActivityStatus::Failed);
}

pub(super) async fn assert_start_control_trace(proof: &ActivityEvidenceProof) {
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
    let status = history_response.status();
    let history_body = response_json(history_response).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "start control trace route should succeed: {history_body}"
    );
    assert!(
        history_body["event_count"].as_u64().unwrap_or_default() > 0,
        "workflow start should record durable control trace: {history_body}"
    );
    let event_kinds = control_history_event_kinds(&history_body);
    assert!(event_kinds.contains(&"run_created"));
    assert!(event_kinds.contains(&"step_created"));
    assert!(event_kinds.contains(&"tool_call_recorded"));

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
    let diagnostics_status = diagnostics_response.status();
    let diagnostics_body = response_json(diagnostics_response).await;
    assert_eq!(
        diagnostics_status,
        StatusCode::OK,
        "start control diagnostics should replay non-empty history: {diagnostics_body}"
    );
}

pub(super) async fn assert_workflow_still_blocked(proof: &ActivityEvidenceProof) {
    let status_response = proof
        .router
        .clone()
        .oneshot(get(format!("/workflows/{}", proof.instance_id).as_str()))
        .await
        .unwrap_or_else(|error| panic!("workflow status route should respond: {error}"));
    let status_body = response_json(status_response).await;
    assert_eq!(status_body["workflow"]["pending_host_work_count"], 1);
}

pub(super) struct ActivityEvidenceProof {
    pub(super) _temp_dir: TempDir,
    pub(super) _valkey: TestValkey,
    pub(super) router: Router,
    pub(super) bpmn_path: PathBuf,
    pub(super) ledger_path: PathBuf,
    pub(super) instance_id: &'static str,
    pub(super) service_token: u64,
}

pub(super) async fn start_mapped_service_evidence_workflow(
    instance_id: &'static str,
) -> ActivityEvidenceProof {
    start_mapped_service_evidence_workflow_with_router(
        instance_id,
        server_router_with_valkey_and_control_ledger,
    )
    .await
}

pub(super) async fn start_mapped_service_evidence_workflow_with_recovery_hot_state(
    instance_id: &'static str,
) -> ActivityEvidenceProof {
    start_mapped_service_evidence_workflow_with_router(
        instance_id,
        server_router_with_valkey_control_ledger_and_in_memory_hot_state,
    )
    .await
}

pub(super) async fn start_mapped_service_evidence_workflow_with_control_ledger_only(
    instance_id: &'static str,
) -> ActivityEvidenceProof {
    start_mapped_service_evidence_workflow_with_router(
        instance_id,
        server_router_with_valkey_control_ledger_only,
    )
    .await
}

pub(super) async fn start_mapped_service_evidence_workflow_with_router(
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

pub(super) fn replay_activity_evidence(
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

pub(super) fn replay_activity_evidence_event_kinds(
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
        .filter_map(|record| match &record.event.kind {
            ControlEventKind::ActivityScheduled { .. } => Some("activity_scheduled"),
            ControlEventKind::ActivityStarted { .. } => Some("activity_started"),
            ControlEventKind::ActivityCompleted { .. } => Some("activity_completed"),
            ControlEventKind::ActivityFailed { .. } => Some("activity_failed"),
            _ => None,
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

pub(super) fn server_router_without_control_ledger(flowhub_root: PathBuf) -> Router {
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
    let ledger = must_ok(
        DuckDbControlLedger::open(&control_ledger_path),
        "control ledger should open for recovery apply test",
    );
    let command = QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some(valkey_url),
        require_valkey_ready: None,
        flowhub_root: Some(flowhub_root),
        control_ledger_path: Some(control_ledger_path),
    };
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
    let ledger = must_ok(
        DuckDbControlLedger::open(&control_ledger_path),
        "control ledger should open for recovery apply hot-state guard test",
    );
    let command = QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some(valkey_url),
        require_valkey_ready: None,
        flowhub_root: Some(flowhub_root),
        control_ledger_path: Some(control_ledger_path),
    };
    qianji_bpmn_workflow_router(
        QianjiBpmnWorkflowHttpState::new(
            build_workflow_control_service(&command),
            QianjiBpmnHostBridge::default(),
        )
        .with_activity_evidence_ledger(Arc::new(ledger)),
    )
}

pub(super) fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("GET request should build: {error}"))
}

pub(super) fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap_or_else(|error| panic!("POST request should build: {error}"))
}

pub(super) async fn response_json(response: axum::response::Response) -> Value {
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
