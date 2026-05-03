use axum::body::to_bytes;
use axum::http::StatusCode;

use super::support::{mismatched_pid, remove_temp_gateway_pidfile, write_temp_gateway_pidfile};
use crate::bin_support::wendao::execute::gateway::health::gateway_health_response;

#[test]
fn test_health_endpoint_reports_process_id_header() {
    let response = gateway_health_response(None);

    assert_eq!(response.status(), StatusCode::OK);
    let process_id = response
        .headers()
        .get("x-wendao-process-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_else(|| panic!("health response should include a process id header"));
    assert_eq!(
        process_id.parse::<u32>().unwrap_or_else(|error| panic!(
            "health response header should be a valid process id: {error}"
        )),
        std::process::id()
    );
}

#[tokio::test]
async fn test_health_endpoint_reports_ready_payload() {
    let response = gateway_health_response(None);

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("health response should be readable: {error}"));
    let payload: serde_json::Value = serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("health response should be valid json: {error}"));
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["ready"], serde_json::json!(true));
    assert_eq!(payload["service"], "wendao-gateway");
    assert_eq!(payload["processId"], serde_json::json!(std::process::id()));
    assert_eq!(payload["planes"]["http"], "ready");
}

#[test]
fn test_health_endpoint_accepts_owned_pidfile() {
    let pidfile = write_temp_gateway_pidfile(&std::process::id().to_string());
    let response = gateway_health_response(Some(pidfile.as_path()));
    remove_temp_gateway_pidfile(&pidfile);

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_endpoint_rejects_mismatched_pidfile() {
    let expected_pid = mismatched_pid();
    let pidfile = write_temp_gateway_pidfile(&expected_pid.to_string());
    let response = gateway_health_response(Some(pidfile.as_path()));
    remove_temp_gateway_pidfile(&pidfile);

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let process_id = response
        .headers()
        .get("x-wendao-process-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_else(|| panic!("health error response should include a process id header"));
    assert_eq!(
        process_id.parse::<u32>().unwrap_or_else(|error| panic!(
            "health error response header should be a valid process id: {error}"
        )),
        std::process::id()
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("health error response should be readable: {error}"));
    let payload: serde_json::Value = serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("health error response should be valid json: {error}"));
    assert_eq!(payload["status"], "degraded");
    assert_eq!(payload["ready"], serde_json::json!(false));
    assert_eq!(payload["service"], "wendao-gateway");
    assert_eq!(payload["error"], "gateway is not ready");
    assert_eq!(payload["expectedPid"], serde_json::json!(expected_pid));
    assert_eq!(payload["processId"], serde_json::json!(std::process::id()));
    assert_eq!(payload["planes"]["http"], "ready");
}
