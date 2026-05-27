use axum::http::StatusCode;
use serde_json::json;
use tower::util::ServiceExt;

use super::support::{
    assert_failure_route_preserves_checkpoint, post_json, response_json, service_failure_payload,
    start_mapped_service_evidence_workflow_with_control_ledger_only,
    start_mapped_service_evidence_workflow_with_recovery_hot_state,
};

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
    assert!(
        body["diagnostics"]["summary"]["event_count"]
            .as_u64()
            .unwrap_or_default()
            >= 5
    );
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
