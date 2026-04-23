#![cfg(feature = "sqlite")]

use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowHttpState,
    QianjiBpmnWorkflowStartHttpRequest, qianji_bpmn_workflow_router,
};
use qianji_bpmn_engine::EventPollOutcome;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;
use tokio::net::TcpListener;

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

#[tokio::test(flavor = "current_thread")]
async fn bpmn_workflow_http_router_starts_and_loads_sqlite_status() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_linear_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("http-start.sqlite3");
    let bpmn_path_json = bpmn_path_to_str(&bpmn_path).to_string();
    let sqlite_path_json = bpmn_path_to_str(&sqlite_path).to_string();
    let base_url = spawn_router(QianjiBpmnHostBridge::default()).await;
    let client = reqwest::Client::new();

    let start = client
        .post(format!("{base_url}/workflows/start"))
        .json(&json!({
            "bpmn_path": bpmn_path_json,
            "process_id": "linear",
            "instance_id": "wf_http_start",
            "initial_variables": { "amount": 7 },
            "checkpoint_backend": {
                "kind": "sqlite",
                "path": sqlite_path_json,
            },
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("start request should send: {error}"));

    assert_eq!(start.status(), reqwest::StatusCode::OK);
    let start_body = start
        .json::<Value>()
        .await
        .unwrap_or_else(|error| panic!("start response should decode: {error}"));
    assert_eq!(start_body["outcome"], "completed");
    assert_eq!(start_body["workflow"]["instance_id"], "wf_http_start");
    assert_eq!(start_body["workflow"]["lifecycle"], "completed");
    assert_eq!(start_body["workflow"]["variables"]["amount"], 7);

    let status = client
        .get(format!("{base_url}/workflows/wf_http_start"))
        .query(&[
            ("checkpoint_backend", "sqlite"),
            (
                "sqlite_path",
                sqlite_path
                    .to_str()
                    .unwrap_or_else(|| panic!("sqlite path should be UTF-8")),
            ),
        ])
        .send()
        .await
        .unwrap_or_else(|error| panic!("status request should send: {error}"));

    assert_eq!(status.status(), reqwest::StatusCode::OK);
    let status_body = status
        .json::<Value>()
        .await
        .unwrap_or_else(|error| panic!("status response should decode: {error}"));
    assert_eq!(status_body["checkpoint_backend"], "sqlite");
    assert_eq!(status_body["workflow"]["instance_id"], "wf_http_start");
    assert_eq!(status_body["workflow"]["lifecycle"], "completed");
}

#[tokio::test(flavor = "current_thread")]
async fn bpmn_workflow_http_router_polls_checkpointed_event_wait() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("http-event.sqlite3");
    let bpmn_path_json = bpmn_path_to_str(&bpmn_path).to_string();
    let sqlite_path_json = bpmn_path_to_str(&sqlite_path).to_string();
    let polls = Arc::new(AtomicUsize::new(0));
    let host = QianjiBpmnHostBridge::builder()
        .on_event_poll({
            let polls = Arc::clone(&polls);
            move |request| {
                let polls = Arc::clone(&polls);
                async move {
                    assert_eq!(request.instance_id, "wf_http_event");
                    let poll_index = polls.fetch_add(1, Ordering::SeqCst);
                    Ok(EventPollOutcome {
                        ready: poll_index > 0,
                        winning_wait_node_index: None,
                        data: if poll_index > 0 {
                            json!({ "approved": true })
                        } else {
                            json!({})
                        },
                    })
                }
            }
        })
        .build();
    let base_url = spawn_router(host).await;
    let client = reqwest::Client::new();

    let start = client
        .post(format!("{base_url}/workflows/start"))
        .json(&json!({
            "bpmn_path": bpmn_path_json,
            "process_id": "wait_flow",
            "instance_id": "wf_http_event",
            "initial_variables": { "amount": 7 },
            "checkpoint_backend": {
                "kind": "sqlite",
                "path": sqlite_path_json,
            },
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("event start request should send: {error}"));
    assert_eq!(start.status(), reqwest::StatusCode::OK);
    let start_body = start
        .json::<Value>()
        .await
        .unwrap_or_else(|error| panic!("event start response should decode: {error}"));
    assert_eq!(start_body["outcome"], "waiting_external_event");
    assert_eq!(start_body["workflow"]["wait_registration_count"], 1);

    let poll = client
        .post(format!("{base_url}/workflows/wf_http_event/events/poll"))
        .json(&json!({
            "bpmn_path": bpmn_path_to_str(&bpmn_path),
            "checkpoint_backend": {
                "kind": "sqlite",
                "path": bpmn_path_to_str(&sqlite_path),
            },
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("event poll request should send: {error}"));
    assert_eq!(poll.status(), reqwest::StatusCode::OK);
    let poll_body = poll
        .json::<Value>()
        .await
        .unwrap_or_else(|error| panic!("event poll response should decode: {error}"));
    assert_eq!(poll_body["outcome"], "completed");
    assert_eq!(poll_body["resumed_from_checkpoint"], true);
    assert_eq!(poll_body["workflow"]["variables"]["approved"], true);
}

async fn spawn_router(host: QianjiBpmnHostBridge) -> String {
    let app = qianji_bpmn_workflow_router(QianjiBpmnWorkflowHttpState::new(
        QianjiBpmnWorkflowControlService::new(),
        host,
    ));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|error| panic!("HTTP test listener should bind: {error}"));
    let address = listener
        .local_addr()
        .unwrap_or_else(|error| panic!("HTTP test listener address should resolve: {error}"));
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .unwrap_or_else(|error| panic!("BPMN HTTP router should serve: {error}"));
    });
    format!("http://{address}")
}

fn write_linear_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("linear.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_linear">
  <bpmn:process id="linear" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

fn write_wait_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("wait.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_wait">
  <bpmn:process id="wait_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:intermediateCatchEvent id="wait_message">
      <bpmn:messageEventDefinition messageRef="invoice_received" name="InvoiceReceived" />
    </bpmn:intermediateCatchEvent>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="wait_message" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="wait_message" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

fn write_file(path: &Path, contents: &str) {
    fs::write(path, contents)
        .unwrap_or_else(|error| panic!("should write BPMN fixture {}: {error}", path.display()));
}

fn bpmn_path_to_str(path: &Path) -> &str {
    path.to_str()
        .unwrap_or_else(|| panic!("BPMN path should be UTF-8"))
}
