use axum::http::StatusCode;
use tower::util::ServiceExt;

use super::support::{get, response_json, start_mapped_service_evidence_workflow};

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
