use axum::http::StatusCode;
use serde_json::json;
use tower::util::ServiceExt;

use super::support::{get, post_json, response_json, start_mapped_service_evidence_workflow};

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_execution_graph_returns_server_owned_element_states() {
    let proof =
        start_mapped_service_evidence_workflow("qianji_server_control_execution_graph_projection")
            .await;

    let response = proof
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/bpmn.workflow.{}/execution-graph",
            proof.instance_id
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("control execution graph route should respond: {error}"));
    let status = response.status();
    let body = response_json(response).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "control execution graph route should succeed: {body}"
    );
    assert_eq!(
        body["run_id"],
        format!("bpmn.workflow.{}", proof.instance_id)
    );
    assert!(
        body["element_count"].as_u64().unwrap_or_default() >= 2,
        "execution graph should include BPMN element states: {body}"
    );
    assert_element_state(&body, "start", "completed");
    assert_element_state(&body, "resolve_project", "active");
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_run_stream_returns_durable_ui_rows() {
    let proof = start_mapped_service_evidence_workflow("qianji_server_control_run_stream").await;

    let response = proof
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/bpmn.workflow.{}/stream",
            proof.instance_id
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("control run stream route should respond: {error}"));
    let status = response.status();
    let body = response_json(response).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "control run stream route should succeed: {body}"
    );
    assert_eq!(
        body["schema_version"],
        "xiuxian_qianji.control.run_stream.v1"
    );
    assert_eq!(
        body["run_id"],
        format!("bpmn.workflow.{}", proof.instance_id)
    );
    assert!(
        body["row_count"].as_u64().unwrap_or_default() >= 4,
        "run stream should include durable BPMN and host-work rows: {body}"
    );
    assert_stream_row(&body, "run_created", "bpmn");
    assert_stream_row(&body, "activity_scheduled", "llm");
    assert_stream_element(&body, "resolve_project");
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_execution_graph_maps_host_evidence_to_bpmn_element_ids() {
    let proof =
        start_mapped_service_evidence_workflow("qianji_server_control_execution_graph_completion")
            .await;
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
        .clone()
        .oneshot(post_json(
            format!("/workflows/{}/tasks/complete", proof.instance_id).as_str(),
            &complete_payload,
        ))
        .await
        .unwrap_or_else(|error| panic!("service completion route should respond: {error}"));
    assert_eq!(complete_response.status(), StatusCode::OK);

    let graph_response = proof
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/bpmn.workflow.{}/execution-graph",
            proof.instance_id
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("control execution graph route should respond: {error}"));
    let graph_body = response_json(graph_response).await;

    assert_element_state(&graph_body, "resolve_project", "completed");
    assert_no_synthetic_activity_elements(&graph_body);
}

fn assert_stream_row(body: &serde_json::Value, kind: &str, source: &str) {
    let rows = body["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("run stream should include rows: {body}"));
    assert!(
        rows.iter()
            .any(|row| row["kind"] == kind && row["source"] == source),
        "run stream should include {source} row {kind}: {body}"
    );
}

fn assert_stream_element(body: &serde_json::Value, element_id: &str) {
    let rows = body["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("run stream should include rows: {body}"));
    assert!(
        rows.iter().any(|row| row["element_id"] == element_id),
        "run stream should pin rows to BPMN element {element_id}: {body}"
    );
}

fn assert_element_state(body: &serde_json::Value, element_id: &str, state: &str) {
    let elements = body["elements"]
        .as_array()
        .unwrap_or_else(|| panic!("execution graph should include elements: {body}"));
    let element = elements
        .iter()
        .find(|element| element["element_id"] == element_id)
        .unwrap_or_else(|| panic!("execution graph should include {element_id}: {body}"));
    assert_eq!(element["state"], state);
    assert!(
        element["source_event_sequence"].as_u64().is_some(),
        "element state should include source event sequence: {element}"
    );
    assert!(
        element["source_event_kind"].as_str().is_some(),
        "element state should include source event kind: {element}"
    );
}

fn assert_no_synthetic_activity_elements(body: &serde_json::Value) {
    let elements = body["elements"]
        .as_array()
        .unwrap_or_else(|| panic!("execution graph should include elements: {body}"));
    let synthetic = elements.iter().find(|element| {
        element["element_id"]
            .as_str()
            .is_some_and(|element_id| element_id.starts_with("bpmn."))
    });
    assert!(
        synthetic.is_none(),
        "execution graph should not expose synthetic activity ids as canvas elements: {body}"
    );
}
