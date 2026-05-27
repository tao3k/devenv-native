use axum::http::StatusCode;
use serde_json::json;
use tempfile::TempDir;
use tower::util::ServiceExt;

use super::support::{get, post_json, response_json, server_router_without_control_ledger};

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
        .oneshot(get("/control/runs/bpmn.workflow.missing/bpmn-source"))
        .await
        .unwrap_or_else(|error| panic!("control BPMN source route should respond: {error}"));
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
