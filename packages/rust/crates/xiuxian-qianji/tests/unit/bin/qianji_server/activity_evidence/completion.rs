use axum::http::StatusCode;
use serde_json::json;
use tower::util::ServiceExt;
use xiuxian_qianji_control::ActivityStatus;

use super::support::{
    assert_start_control_trace, post_json, replay_activity_evidence,
    start_mapped_service_evidence_workflow_with_control_ledger_only,
};

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_completion_records_host_work_activity_evidence() {
    let proof = start_mapped_service_evidence_workflow_with_control_ledger_only(
        "qianji_server_host_work_activity_evidence",
    )
    .await;
    assert_start_control_trace(&proof).await;

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
            "activity_scheduled",
            "activity_started",
            "activity_completed",
        ]
    );
    assert_eq!(activity_status, ActivityStatus::Completed);
}
