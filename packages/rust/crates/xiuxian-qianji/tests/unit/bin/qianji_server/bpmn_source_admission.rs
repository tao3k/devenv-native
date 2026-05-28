use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tower::util::ServiceExt;

use crate::runtime_config::QianjiRuntimeEnv;
use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowHttpState,
    qianji_bpmn_workflow_router,
};

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_admits_valid_bpmn_source_under_server_cache() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let router = server_router(temp_dir.path());

    let response = router
        .oneshot(post_json(
            "/control/bpmn-source/admit",
            &json!({
                "source_id": "wendao ai/example run",
                "process_id": "mapped_service_boundary",
                "bpmn_xml": MAPPED_SERVICE_BOUNDARY_BPMN,
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("admission route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["source_id"], "wendao_ai_example_run");
    assert_eq!(body["process_id"], "mapped_service_boundary");
    assert_eq!(body["media_type"], "application/bpmn+xml");
    assert_eq!(body["lint_issue_count"], 0);
    let bpmn_path = PathBuf::from(
        body["bpmn_path"]
            .as_str()
            .unwrap_or_else(|| panic!("response should include bpmn_path: {body}")),
    );
    assert!(
        bpmn_path.starts_with(temp_dir.path().join(".cache/qianji/bpmn-sources")),
        "admitted source should be server-owned under the configured project cache: {body}",
    );
    assert_eq!(
        fs::read_to_string(&bpmn_path)
            .unwrap_or_else(|error| panic!("admitted BPMN should be readable: {error}")),
        MAPPED_SERVICE_BOUNDARY_BPMN,
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_rejects_bpmn_admission_when_process_id_is_missing() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let router = server_router(temp_dir.path());

    let response = router
        .oneshot(post_json(
            "/control/bpmn-source/admit",
            &json!({
                "source_id": "bad-process",
                "process_id": "missing_process",
                "bpmn_xml": MAPPED_SERVICE_BOUNDARY_BPMN,
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("admission route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["code"], "bpmn_source_process_missing");
    assert!(
        !temp_dir.path().join(".cache/qianji/bpmn-sources").exists(),
        "rejected sources must not be written into the server cache",
    );
}

fn server_router(project_root: &Path) -> Router {
    let runtime_env = QianjiRuntimeEnv {
        prj_root: Some(project_root.to_path_buf()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env.clone());
    let state = QianjiBpmnWorkflowHttpState::new(service, QianjiBpmnHostBridge::default())
        .with_runtime_env(runtime_env);
    qianji_bpmn_workflow_router(state)
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

const MAPPED_SERVICE_BOUNDARY_BPMN: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_mapped_service_boundary">
  <bpmn:process id="mapped_service_boundary" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="resolve_project" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_resolve" sourceRef="start" targetRef="resolve_project" />
    <bpmn:sequenceFlow id="flow_resolve_done" sourceRef="resolve_project" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#;
