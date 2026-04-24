use crate::{
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowHttpCheckpointBackend,
    QianjiBpmnWorkflowStartHttpRequest,
};
use serde_json::json;

#[test]
fn bpmn_workflow_http_requests_default_to_runtime_valkey_backend() {
    let start = serde_json::from_value::<QianjiBpmnWorkflowStartHttpRequest>(json!({
        "bpmn_path": "flow.bpmn",
        "process_id": "flow",
        "instance_id": "wf_http_default",
    }))
    .unwrap_or_else(|error| panic!("start HTTP request should decode: {error}"));
    assert_eq!(
        start.checkpoint_backend,
        QianjiBpmnWorkflowHttpCheckpointBackend::RuntimeValkey
    );

    let action = serde_json::from_value::<QianjiBpmnWorkflowActionHttpRequest>(json!({
        "bpmn_path": "flow.bpmn",
    }))
    .unwrap_or_else(|error| panic!("action HTTP request should decode: {error}"));
    assert_eq!(
        action.checkpoint_backend,
        QianjiBpmnWorkflowHttpCheckpointBackend::RuntimeValkey
    );
}

#[test]
fn bpmn_workflow_http_rejects_local_duckdb_backend_contract() {
    let error = match serde_json::from_value::<QianjiBpmnWorkflowHttpCheckpointBackend>(json!({
        "kind": "duckdb",
        "path": "state.duckdb",
    })) {
        Ok(backend) => {
            panic!("HTTP checkpoint backend should reject local DuckDB kind: {backend:?}")
        }
        Err(error) => error,
    };

    assert!(
        error.to_string().contains("unknown variant `duckdb`"),
        "unexpected decode error: {error}"
    );
}
