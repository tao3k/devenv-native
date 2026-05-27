use std::path::Path;
use std::sync::Arc;
#[cfg(feature = "valkey")]
use std::time::Duration;

use super::support::must_ok;
use crate::qianji_server::llm_worker::{
    QianjiServerOpenAiCompatibleLlmWorkerLoopRequest,
    run_qianji_server_openai_compatible_llm_worker_loop,
};
#[cfg(feature = "valkey")]
use crate::qianji_test_valkey_support::TestValkey;
use crate::runtime_config::QianjiRuntimeEnv;
use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowHttpState,
    qianji_bpmn_workflow_router,
};
use axum::Router;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, AdmittedLlmActivityScheduleRecord, ArtifactId,
    ArtifactKind, ArtifactRef, ControlLedger, HotStateStore, IdempotencyKey, InMemoryControlLedger,
    InMemoryHotStateStore, LlmActivityAdmission, LlmActivityRequest, LlmActivityTask, LlmModelId,
    RunCreatedJournalRecord, RunId, TaskQueue, record_admitted_llm_activity_schedule_idempotent,
    record_run_created,
};
#[cfg(feature = "valkey")]
use xiuxian_qianji_control::{RunnableActivityTask, WorkerActivityTask};
#[cfg(feature = "valkey")]
use xiuxian_qianji_control::{ValkeyHotStateConfig, ValkeyHotStateStore};

#[tokio::test]
async fn qianji_server_llm_worker_loop_executes_openai_compatible_activity() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("temporary directory should allocate: {error}"))?;
    let prompt_path = temp_dir.path().join("prompt.txt");
    std::fs::write(&prompt_path, "Summarize qianji server durable state.")
        .map_err(|error| format!("prompt fixture should write: {error}"))?;
    let output_dir = temp_dir.path().join("llm-output");
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"Server-owned durable LLM result."}}]}"#,
    )
    .await?;
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = seed_openai_compatible_llm_activity(
        &ledger,
        &prompt_path,
        "activity-qianji-server-openai-compatible-loop",
    )?;

    let output = run_qianji_server_openai_compatible_llm_worker_loop(
        &ledger,
        &hot_state,
        QianjiServerOpenAiCompatibleLlmWorkerLoopRequest {
            run_id: &run_id,
            worker_id: "qianji-server-llm-worker",
            task_queue: Some("llm.openrouter"),
            now_ms: 8_000,
            now_step_ms: 1,
            lease_ttl_ms: 500,
            heartbeat_ttl_ms: None,
            poll_limit: 2,
            empty_limit: 1,
            worker_count: 1,
            settled_at_ms: 9_000,
            settled_step_ms: 1,
            openai_compatible_base_url: base_url.as_str(),
            openai_compatible_api_key: Some("server-key"),
            openai_compatible_timeout_ms: Some(5_000),
            output_artifact_dir: output_dir.as_path(),
            output_artifact_kind: Some("llm.response"),
        },
    )
    .await
    .map_err(|error| format!("qianji-server LLM worker should run: {error}"))?;
    let request = request_rx
        .await
        .map_err(|error| format!("provider request should be captured: {error}"))?;
    let artifact_path = output_dir
        .join("activity-qianji-server-openai-compatible-loop-attempt-1.openai-compatible-llm.json");
    let artifact_json: serde_json::Value = must_ok(
        serde_json::from_str(
            &std::fs::read_to_string(&artifact_path)
                .map_err(|error| format!("provider artifact should read: {error}"))?,
        ),
        "provider artifact should decode",
    );
    let queue = TaskQueue::new("llm.openrouter")
        .map_err(|error| format!("task queue should build: {error}"))?;
    let projection = ledger
        .load_activity_queue_projection(&run_id, Some(&queue))
        .map_err(|error| format!("queue projection should load: {error}"))?;
    let snapshot = hot_state
        .load_snapshot(9_100)
        .await
        .map_err(|error| format!("hot-state snapshot should load: {error}"))?;

    assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(request.contains("authorization: Bearer server-key"));
    assert_eq!(output.processed, 1);
    assert_eq!(output.released, 1);
    assert_eq!(output.stopped_reason, "EmptyLimit");
    assert_eq!(
        output.iterations[0].activity_id.as_deref(),
        Some("activity-qianji-server-openai-compatible-loop")
    );
    assert!(output.iterations[0].terminal_recorded);
    assert_eq!(projection.summary.completed, 1);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    assert_eq!(
        artifact_json["schema"],
        "qianji.openai_compatible_llm_response.v1"
    );
    assert_eq!(artifact_json["content"], "Server-owned durable LLM result.");
    Ok(())
}

#[tokio::test]
async fn qianji_server_llm_worker_http_route_executes_openai_compatible_activity()
-> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("temporary directory should allocate: {error}"))?;
    let prompt_path = temp_dir.path().join("prompt.txt");
    std::fs::write(&prompt_path, "Summarize qianji server HTTP worker state.")
        .map_err(|error| format!("prompt fixture should write: {error}"))?;
    let output_dir = temp_dir.path().join("http-llm-output");
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"HTTP route durable LLM result."}}]}"#,
    )
    .await?;
    let ledger = Arc::new(InMemoryControlLedger::new());
    let hot_state = Arc::new(InMemoryHotStateStore::new());
    let run_id = seed_openai_compatible_llm_activity(
        ledger.as_ref(),
        &prompt_path,
        "activity-qianji-server-http-openai-compatible-loop",
    )?;
    let control_ledger: Arc<dyn ControlLedger> = ledger.clone();
    let control_hot_state: Arc<dyn HotStateStore> = hot_state.clone();
    let router = qianji_bpmn_workflow_router(
        QianjiBpmnWorkflowHttpState::new(
            QianjiBpmnWorkflowControlService::new(),
            QianjiBpmnHostBridge::default(),
        )
        .with_activity_evidence_ledger(control_ledger)
        .with_recovery_hot_state(control_hot_state)
        .with_runtime_env(QianjiRuntimeEnv {
            prj_root: Some(temp_dir.path().join("project-root")),
            prj_config_home: Some(temp_dir.path().join("config-home")),
            openai_api_base: Some(base_url.clone()),
            openai_api_key: Some("server-key".to_owned()),
            qianji_llm_model: Some("openrouter/qwen/qwen3-coder".to_owned()),
            ..QianjiRuntimeEnv::default()
        }),
    );
    let server_url = spawn_http_router(router).await?;

    let response = reqwest::Client::new()
        .post(format!(
            "{server_url}/control/runs/{}/workers/openai-compatible-llm/run",
            run_id.as_str()
        ))
        .json(&serde_json::json!({
            "worker_id": "qianji-server-http-llm-worker",
            "task_queue": "llm.openrouter",
            "now_ms": 8_000,
            "now_step_ms": 1,
            "lease_ttl_ms": 500,
            "poll_limit": 2,
            "empty_limit": 1,
            "worker_count": 1,
            "settled_at_ms": 9_000,
            "settled_step_ms": 1,
            "output_artifact_dir": output_dir.display().to_string(),
            "output_artifact_kind": "llm.response",
            "openai_compatible_timeout_ms": 5_000
        }))
        .send()
        .await
        .map_err(|error| format!("HTTP worker route should respond: {error}"))?;
    let status = response.status();
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|error| format!("HTTP worker route JSON should decode: {error}"))?;
    let request = request_rx
        .await
        .map_err(|error| format!("provider request should be captured: {error}"))?;
    let artifact_path = output_dir.join(
        "activity-qianji-server-http-openai-compatible-loop-attempt-1.openai-compatible-llm.json",
    );
    let artifact_json: serde_json::Value = must_ok(
        serde_json::from_str(
            &std::fs::read_to_string(&artifact_path)
                .map_err(|error| format!("provider artifact should read: {error}"))?,
        ),
        "provider artifact should decode",
    );

    assert_eq!(status, reqwest::StatusCode::OK, "{response_json}");
    assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(request.contains("authorization: Bearer server-key"));
    assert_eq!(response_json["run_id"], run_id.as_str());
    assert_eq!(response_json["worker"]["processed"], 1);
    assert_eq!(response_json["worker"]["released"], 1);
    assert_eq!(response_json["worker"]["stoppedReason"], "EmptyLimit");
    assert_eq!(artifact_json["content"], "HTTP route durable LLM result.");
    Ok(())
}

#[cfg(feature = "valkey")]
#[tokio::test]
async fn qianji_server_workflow_start_schedules_toml_configured_llm_activity() -> Result<(), String>
{
    let valkey = TestValkey::spawn()
        .await
        .map_err(|error| format!("valkey should start for workflow LLM bridge: {error}"))?;
    let temp_dir =
        TempDir::new().map_err(|error| format!("temporary directory should allocate: {error}"))?;
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"{\"resolvedProject\":true}"}}]}"#,
    )
    .await?;
    let project_root = temp_dir.path().join("project-root");
    write_qianji_runtime_config_fixture(&project_root, base_url.as_str())?;
    let bpmn_path = write_llm_service_boundary_bpmn(temp_dir.path())?;
    let output_dir = temp_dir.path().join("llm-output");
    let ledger = Arc::new(InMemoryControlLedger::new());
    let hot_state_config = ValkeyHotStateConfig::new(valkey.url())
        .map_err(|error| format!("valkey hot-state config should build: {error}"))?
        .with_namespace("qianji:test:workflow-start-llm")
        .map_err(|error| format!("valkey hot-state namespace should build: {error}"))?;
    let hot_state = Arc::new(ValkeyHotStateStore::new(hot_state_config));
    enqueue_stale_openai_compatible_activity(hot_state.as_ref()).await?;
    let control_ledger: Arc<dyn ControlLedger> = ledger.clone();
    let control_hot_state: Arc<dyn HotStateStore> = hot_state.clone();
    let router = qianji_bpmn_workflow_router(
        QianjiBpmnWorkflowHttpState::new(
            QianjiBpmnWorkflowControlService::new().with_runtime_env(QianjiRuntimeEnv {
                qianji_checkpoint_valkey_url: Some(valkey.url().to_string()),
                ..QianjiRuntimeEnv::default()
            }),
            QianjiBpmnHostBridge::default(),
        )
        .with_activity_evidence_ledger(control_ledger)
        .with_recovery_hot_state(control_hot_state)
        .with_runtime_env(QianjiRuntimeEnv {
            prj_root: Some(project_root),
            prj_config_home: Some(temp_dir.path().join("config-home")),
            extra_env: vec![("DEEPSEEK_API_KEY".to_owned(), "server-key".to_owned())],
            ..QianjiRuntimeEnv::default()
        }),
    );
    let server_url = spawn_http_router(router).await?;
    let instance_id = "qianji_server_llm_start_bridge";

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|error| format!("HTTP client should build: {error}"))?;

    let start_response = client
        .post(format!("{server_url}/workflows/start"))
        .json(&serde_json::json!({
            "bpmn_path": bpmn_path.display().to_string(),
            "process_id": "llm_service_boundary",
            "instance_id": instance_id,
            "initial_variables": {
                "brief": "Use qianji-server TOML routed LLM."
            }
        }))
        .send()
        .await
        .map_err(|error| format!("workflow start route should respond: {error}"))?;
    let start_status = start_response.status();
    let start_json: serde_json::Value = start_response
        .json()
        .await
        .map_err(|error| format!("workflow start JSON should decode: {error}"))?;
    assert_eq!(start_status, reqwest::StatusCode::OK, "{start_json}");

    let run_id = format!("bpmn.workflow.{instance_id}");
    let worker_response = client
        .post(format!(
            "{server_url}/control/runs/{run_id}/workers/openai-compatible-llm/run-and-complete"
        ))
        .json(&serde_json::json!({
            "worker_id": "qianji-server-start-llm-worker",
            "now_ms": 8_000,
            "now_step_ms": 1,
            "lease_ttl_ms": 500,
            "poll_limit": 2,
            "empty_limit": 1,
            "worker_count": 1,
            "settled_at_ms": 9_000,
            "settled_step_ms": 1,
            "output_artifact_dir": output_dir.display().to_string(),
            "output_artifact_kind": "llm.response",
            "openai_compatible_timeout_ms": 5_000
        }))
        .send()
        .await
        .map_err(|error| format!("HTTP worker route should respond: {error}"))?;
    let worker_status = worker_response.status();
    let worker_json: serde_json::Value = worker_response
        .json()
        .await
        .map_err(|error| format!("worker route JSON should decode: {error}"))?;
    assert_eq!(worker_status, reqwest::StatusCode::OK, "{worker_json}");
    assert_eq!(worker_json["completed_count"], 1, "{worker_json}");
    assert_eq!(
        worker_json["worker_runs"][0]["processed"], 1,
        "{worker_json}"
    );
    assert_eq!(
        worker_json["worker_runs"][0]["iterations"][0]["activityId"],
        "bpmn-llm-qianji_server_llm_start_bridge-resolve_project-1"
    );
    assert_eq!(
        worker_json["final_workflow"]["workflow"]["pending_host_work_count"], 0,
        "{worker_json}"
    );
    assert_eq!(
        worker_json["final_workflow"]["workflow"]["variables"]["resolvedProject"], true,
        "{worker_json}"
    );
    let provider_request = tokio::time::timeout(Duration::from_secs(10), request_rx)
        .await
        .map_err(|_| "provider request should be captured before timeout".to_owned())?
        .map_err(|error| format!("provider request should be captured: {error}"))?;

    assert!(provider_request.contains("authorization: Bearer server-key"));
    assert!(provider_request.contains("\"model\":\"deepseek-test\""));
    assert!(provider_request.contains("qianji_bpmn_host_work"));
    assert!(provider_request.contains("resolvedProject"));
    Ok(())
}

fn seed_openai_compatible_llm_activity(
    ledger: &impl ControlLedger,
    prompt_path: &Path,
    activity_id: &str,
) -> Result<RunId, String> {
    let run_id = RunId::new("run-qianji-server-llm-worker").map_err(|error| format!("{error}"))?;
    record_run_created(
        ledger,
        RunCreatedJournalRecord::new(run_id.clone(), "qianji-server LLM worker proof", 1),
    )
    .map_err(|error| format!("run-created event should record: {error}"))?;
    let prompt_ref = ArtifactRef {
        artifact_id: ArtifactId::new("artifact-qianji-server-llm-prompt")
            .map_err(|error| format!("artifact id should build: {error}"))?,
        artifact_kind: ArtifactKind::new("llm.prompt")
            .map_err(|error| format!("artifact kind should build: {error}"))?,
        uri: prompt_path.display().to_string(),
        content_digest: None,
        metadata: serde_json::Value::Null,
    };
    let task = ActivityTask::new(
        ActivityId::new(activity_id)
            .map_err(|error| format!("activity id should build: {error}"))?,
        ActivityType::new("llm.plan")
            .map_err(|error| format!("activity type should build: {error}"))?,
        TaskQueue::new("llm.openrouter")
            .map_err(|error| format!("task queue should build: {error}"))?,
        IdempotencyKey::new(format!("run/{activity_id}/llm"))
            .map_err(|error| format!("idempotency key should build: {error}"))?,
    )
    .with_input_ref(prompt_ref.clone())
    .with_timeout_ms(30_000);
    let request = LlmActivityRequest::new(
        LlmModelId::new("openrouter/qwen/qwen3-coder")
            .map_err(|error| format!("model id should build: {error}"))?,
        prompt_ref,
    )
    .with_temperature_millis(0)
    .with_max_tokens(1_024);
    let admission = LlmActivityAdmission::from_activity(LlmActivityTask::new(task, request))
        .map_err(|error| format!("LLM admission should validate: {error}"))?;
    record_admitted_llm_activity_schedule_idempotent(
        ledger,
        AdmittedLlmActivityScheduleRecord::run(run_id.clone(), 2, admission),
    )
    .map_err(|error| format!("LLM schedule should record: {error}"))?;
    Ok(run_id)
}

#[cfg(feature = "valkey")]
async fn enqueue_stale_openai_compatible_activity(
    hot_state: &impl HotStateStore,
) -> Result<(), String> {
    hot_state
        .enqueue_activity_task(RunnableActivityTask {
            task: WorkerActivityTask {
                run_id: RunId::new("stale-run-without-history")
                    .map_err(|error| format!("stale run id should build: {error}"))?,
                step_id: None,
                activity_id: ActivityId::new("stale-llm-activity")
                    .map_err(|error| format!("stale activity id should build: {error}"))?,
                activity_type: ActivityType::new("llm.plan")
                    .map_err(|error| format!("stale activity type should build: {error}"))?,
                task_queue: TaskQueue::new("llm.deepseek-test")
                    .map_err(|error| format!("stale task queue should build: {error}"))?,
                next_attempt: 1,
                scheduled_at_ms: 1,
                input_ref: None,
                idempotency_key: IdempotencyKey::new("stale-run-without-history/llm")
                    .map_err(|error| format!("stale idempotency key should build: {error}"))?,
                retry_policy: None,
                timeout_ms: Some(30_000),
                metadata: serde_json::Value::Null,
            },
            priority: 99,
            not_before_ms: 0,
            metadata: serde_json::json!({"fixture": "stale-cross-run-task"}),
        })
        .await
        .map_err(|error| format!("stale activity should enqueue: {error}"))
}

#[cfg(feature = "valkey")]
fn write_qianji_runtime_config_fixture(project_root: &Path, base_url: &str) -> Result<(), String> {
    let config_root = project_root.join("packages/rust/crates/xiuxian-qianji/resources/config");
    let workflows_root = config_root.join("workflows");
    std::fs::create_dir_all(&workflows_root)
        .map_err(|error| format!("qianji config fixture directory should create: {error}"))?;
    std::fs::write(
        config_root.join("qianji.toml"),
        format!(
            r#"[llm]
model = "deepseek-test"
base_url = "{base_url}"
api_key_env = "DEEPSEEK_API_KEY"
"#
        ),
    )
    .map_err(|error| format!("qianji.toml fixture should write: {error}"))?;
    std::fs::write(
        workflows_root.join("bpmn-host-work-llm.toml"),
        r#"schema = "qianji.workflow.llm_task.v1"

[llm]
provider = "openai-compatible"
wire_api = "chat_completions"

[task]
activity_type = "llm.plan"
task_queue = "llm.deepseek-test"
idempotency_key_prefix = "qianji:test:bpmn:llm"
prompt_artifact_kind = "qianji.bpmn.host_work.prompt"
temperature_millis = 0
max_tokens = 256
timeout_ms = 30000
"#,
    )
    .map_err(|error| format!("workflow LLM config fixture should write: {error}"))?;
    Ok(())
}

#[cfg(feature = "valkey")]
fn write_llm_service_boundary_bpmn(root: &Path) -> Result<std::path::PathBuf, String> {
    let bpmn_path = root.join("llm-service-boundary.bpmn");
    std::fs::write(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_llm_service_boundary">
  <bpmn:process id="llm_service_boundary" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="resolve_project">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="resolve_project_output_resolvedProject" name="resolvedProject" />
        <bpmn:outputSet id="resolve_project_output_set">
          <bpmn:dataOutputRefs>resolve_project_output_resolvedProject</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>resolve_project_output_resolvedProject</bpmn:sourceRef>
        <bpmn:targetRef>resolvedProject</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_resolve" sourceRef="start" targetRef="resolve_project" />
    <bpmn:sequenceFlow id="flow_resolve_done" sourceRef="resolve_project" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    )
    .map_err(|error| format!("BPMN fixture should write: {error}"))?;
    Ok(bpmn_path)
}

async fn spawn_openai_compatible_server(
    status: &'static str,
    body: &'static str,
) -> Result<(String, tokio::sync::oneshot::Receiver<String>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("test provider server should bind: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("test provider address should resolve: {error}"))?;
    let (request_tx, request_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        if let Ok((mut stream, _peer)) = listener.accept().await
            && let Ok(request) = read_http_request(&mut stream).await
        {
            let _ = request_tx.send(request);
            let response = format!(
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    Ok((format!("http://{address}/v1"), request_rx))
}

async fn spawn_http_router(router: Router) -> Result<String, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("HTTP router should bind: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("HTTP router address should resolve: {error}"))?;
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    Ok(format!("http://{address}"))
}

async fn read_http_request(stream: &mut tokio::net::TcpStream) -> Result<String, std::io::Error> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if request_complete(&buffer) {
            break;
        }
    }
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn request_complete(buffer: &[u8]) -> bool {
    let Some(header_end) = find_header_end(buffer) else {
        return false;
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length: "))
        .or_else(|| {
            headers
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
        })
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    buffer.len().saturating_sub(header_end + 4) >= content_length
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}
