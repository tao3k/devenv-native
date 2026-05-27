use axum::http::StatusCode;
use serde_json::json;
use tower::util::ServiceExt;

use super::support::{
    assert_failed_activity_evidence, assert_failed_control_diagnostics,
    assert_failed_control_history, assert_failed_control_recovery, assert_failed_control_summary,
    assert_failure_route_preserves_checkpoint, assert_start_control_trace,
    assert_workflow_still_blocked, post_json, replay_activity_evidence_event_kinds,
    service_failure_payload, start_mapped_service_evidence_workflow,
};

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_failure_records_host_work_activity_evidence_without_advancing() {
    let proof =
        start_mapped_service_evidence_workflow("qianji_server_host_work_failure_evidence").await;
    assert_start_control_trace(&proof).await;
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
