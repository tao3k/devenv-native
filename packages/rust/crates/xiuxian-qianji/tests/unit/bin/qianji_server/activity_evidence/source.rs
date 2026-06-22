use axum::http::StatusCode;
use tower::util::ServiceExt;

use super::support::{get, response_json, start_mapped_service_evidence_workflow};

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_control_bpmn_source_returns_server_recorded_xml() {
    let proof =
        start_mapped_service_evidence_workflow("qianji_server_control_bpmn_source_ref").await;

    let response = proof
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/bpmn.workflow.{}/bpmn-source",
            proof.instance_id
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("control BPMN source route should respond: {error}"));
    let status = response.status();
    let body = response_json(response).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "control BPMN source route should succeed: {body}"
    );
    assert_eq!(
        body["run_id"],
        format!("bpmn.workflow.{}", proof.instance_id)
    );
    assert_eq!(body["source_ref"], proof.bpmn_path.display().to_string());
    assert_eq!(body["media_type"], "application/bpmn+xml");
    assert!(
        body["bpmn_xml"]
            .as_str()
            .unwrap_or_default()
            .contains(r#"<bpmn:process id="mapped_service_boundary""#),
        "server-owned BPMN source should return the original BPMN XML: {body}"
    );
}
