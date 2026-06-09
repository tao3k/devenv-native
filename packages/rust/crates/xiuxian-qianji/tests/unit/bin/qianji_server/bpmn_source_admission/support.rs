use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, header::CONTENT_TYPE},
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tempfile::TempDir;
use tower::util::ServiceExt;
use xiuxian_qianji_control::{
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore, RunId,
};

use crate::qianji_test_valkey_support::TestValkey;
use crate::runtime_config::QianjiRuntimeEnv;
use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowHttpState,
    qianji_bpmn_workflow_router,
};

pub(super) const WORKFLOW_SOURCE_REPAIR_BPMN: &str =
    include_str!("../../../../../resources/workflows/workflow_source_repair_v1.bpmn");

pub(super) struct RepairRunFixture {
    pub(super) router: Router,
    pub(super) _temp_dir: TempDir,
    pub(super) _valkey: TestValkey,
    pub(super) ledger_path: PathBuf,
    pub(super) run_id: RunId,
    pub(super) instance_id: String,
    pub(super) bpmn_path: String,
}

pub(super) async fn start_repair_lint_flow() -> RepairRunFixture {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let ledger_path = temp_dir.path().join("control-ledger.duckdb");
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start: {error}"));
    let router = server_router_with_repair_runtime(temp_dir.path(), valkey.url().to_string());
    let response = router
        .clone()
        .oneshot(post_json(
            "/control/workflow-source/admit",
            &json!({
                "source_id": "daily report/repair lint",
                "process_id": "Process_wf_repair_lint",
                "source_media_type": "text/markdown",
                "source_text": "# Daily Report Generator\n\nGather metrics and write a report.",
                "workflow_name": "Daily Report Generator",
                "workflow_description": "Repair this free-form workflow source.",
                "compiler_mode": "server_repair",
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("workflow-source admission route should respond: {error}"));

    assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
    let body = response_json(response).await;
    let run_id = RunId::new(
        body["repair_run"]["run_id"]
            .as_str()
            .unwrap_or_else(|| panic!("repair response should include run id: {body}")),
    )
    .unwrap_or_else(|error| panic!("run id should be valid: {error}"));
    let instance_id = body["repair_run"]["instance_id"]
        .as_str()
        .unwrap_or_else(|| panic!("repair response should include instance id: {body}"))
        .to_owned();
    let bpmn_path = body["repair_run"]["bpmn_path"]
        .as_str()
        .unwrap_or_else(|| panic!("repair response should include bpmn path: {body}"))
        .to_owned();

    RepairRunFixture {
        router,
        _temp_dir: temp_dir,
        _valkey: valkey,
        ledger_path,
        run_id,
        instance_id,
        bpmn_path,
    }
}

pub(super) async fn complete_repair_service_task(
    repair: &RepairRunFixture,
    token_id: u64,
    activity_id: &str,
    data: Value,
) -> Value {
    let response = repair
        .router
        .clone()
        .oneshot(post_json(
            format!("/workflows/{}/tasks/complete", repair.instance_id).as_str(),
            &json!({
                "bpmn_path": repair.bpmn_path,
                "completion": {
                    "token_id": token_id,
                    "process_id": "qianji_workflow_source_repair_v1",
                    "activity_id": activity_id,
                    "kind": "service",
                    "data": data,
                },
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("repair completion route should respond: {error}"));
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    response_json(response).await
}

pub(super) fn llm_activity_token(repair: &RepairRunFixture, activity_id: &str) -> u64 {
    let inventory = repair_llm_inventory(repair);
    let activity = inventory
        .items
        .iter()
        .find(|item| item.request_audit_metadata["request_metadata"]["activity_id"] == activity_id)
        .unwrap_or_else(|| panic!("LLM activity `{activity_id}` should be scheduled"));
    activity.request_audit_metadata["request_metadata"]["token_id"]
        .as_u64()
        .unwrap_or_else(|| panic!("LLM activity `{activity_id}` token should be recorded"))
}

fn repair_llm_inventory(
    repair: &RepairRunFixture,
) -> xiuxian_qianji_control::LlmActivityInventoryProjection {
    let ledger = DuckDbControlLedger::open(&repair.ledger_path)
        .unwrap_or_else(|error| panic!("control ledger should reopen: {error}"));
    ledger
        .load_llm_activity_inventory_projection(&repair.run_id)
        .unwrap_or_else(|error| panic!("LLM inventory should project: {error}"))
}

pub(super) async fn repair_control_history(repair: &RepairRunFixture) -> Value {
    let response = repair
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/{}/history",
            repair.run_id.as_str()
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("repair history route should respond: {error}"));
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    response_json(response).await
}

pub(super) fn server_router(project_root: &Path) -> Router {
    let runtime_env = QianjiRuntimeEnv {
        prj_root: Some(project_root.to_path_buf()),
        openai_api_base: Some("http://127.0.0.1:1/v1".to_string()),
        openai_api_key: Some("qianji-server-test-key".to_string()),
        qianji_llm_model: Some("openai-compatible/qianji-test-model".to_string()),
        qianji_llm_wire_api: Some("chat_completions".to_string()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env.clone());
    let state = QianjiBpmnWorkflowHttpState::new(service, QianjiBpmnHostBridge::default())
        .with_runtime_env(runtime_env);
    qianji_bpmn_workflow_router(state)
}

pub(super) fn server_router_with_repair_runtime(project_root: &Path, valkey_url: String) -> Router {
    let runtime_env = QianjiRuntimeEnv {
        prj_root: Some(project_root.to_path_buf()),
        qianji_checkpoint_valkey_url: Some(valkey_url),
        openai_api_base: Some("http://127.0.0.1:1/v1".to_string()),
        openai_api_key: Some("qianji-server-test-key".to_string()),
        qianji_llm_model: Some("openai-compatible/qianji-test-model".to_string()),
        qianji_llm_wire_api: Some("chat_completions".to_string()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env.clone());
    let ledger = DuckDbControlLedger::open(project_root.join("control-ledger.duckdb"))
        .unwrap_or_else(|error| panic!("control ledger should open: {error}"));
    let control_ledger: Arc<dyn ControlLedger> = Arc::new(ledger);
    let hot_state: Arc<dyn HotStateStore> = Arc::new(InMemoryHotStateStore::new());
    let state = QianjiBpmnWorkflowHttpState::new(service, QianjiBpmnHostBridge::default())
        .with_activity_evidence_ledger(control_ledger)
        .with_recovery_hot_state(hot_state)
        .with_runtime_env(runtime_env);
    qianji_bpmn_workflow_router(state)
}

pub(super) fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap_or_else(|error| panic!("POST request should build: {error}"))
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("GET request should build: {error}"))
}

pub(super) async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("response body should decode as JSON: {error}"))
}

pub(super) fn repair_candidate_bpmn(process_id: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_{process_id}">
  <bpmn:process id="{process_id}" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="step-1" name="Gather Inputs" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_step_1" sourceRef="start" targetRef="step-1" />
    <bpmn:sequenceFlow id="flow_step_1_done" sourceRef="step-1" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
    )
}

pub(super) const MAPPED_SERVICE_BOUNDARY_BPMN: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_mapped_service_boundary">
  <bpmn:process id="mapped_service_boundary" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="resolve_project" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_resolve" sourceRef="start" targetRef="resolve_project" />
    <bpmn:sequenceFlow id="flow_resolve_done" sourceRef="resolve_project" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#;
