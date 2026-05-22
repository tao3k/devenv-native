use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerLoopStoreRequest,
    worker_loop_with_hot_state,
};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::tests::control_cli::support::must_some;
use crate::qianji_cli::tests::control_cli::support::{
    activity_task, append_control_run_with_scheduled_activity_queue, append_empty_control_run,
    must_ok,
};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use xiuxian_qianji_control::{
    ActivityId, ArtifactId, ArtifactKind, ArtifactRef, ControlEvent, ControlEventKind,
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore, RunnableActivityTask,
};

#[tokio::test]
async fn worker_loop_with_hot_state_processes_tasks_until_empty_limit() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-run-scheduled").await?;
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-step-scheduled").await?;

    let output = must_ok(
        worker_loop_with_hot_state(
            &ledger,
            &hot_state,
            ActivityWorkerLoopStoreRequest {
                worker_id: "worker-loop",
                task_queue: None,
                now_ms: 8_000,
                now_step_ms: 10,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                poll_limit: 3,
                empty_limit: 1,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                settled_step_ms: 10,
                output_hash: Some("sha256:activity-output"),
                output_artifact_dir: None,
                output_artifact_kind: None,
                openai_compatible_base_url: None,
                openai_compatible_api_key: None,
                openai_compatible_timeout_ms: None,
                error_code: None,
                message: None,
                retryable: None,
                metadata: Some("{\"rows\":3}"),
                json: true,
            },
        )
        .await,
        "activity worker loop should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker loop output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker loop should replay into queue projection",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(9_100).await,
        "hot state snapshot should load after worker loop",
    );

    assert_eq!(json["worker_id"], "worker-loop");
    assert_eq!(json["processed"], 2);
    assert_eq!(json["empty_polls"], 1);
    assert_eq!(json["released"], 2);
    assert_eq!(json["heartbeats"], 0);
    assert_eq!(json["stopped_reason"], "empty_limit");
    assert_eq!(json["iterations"].as_array().map(Vec::len), Some(3));
    assert_eq!(json["iterations"][0]["now_ms"], 8_000);
    assert_eq!(json["iterations"][1]["now_ms"], 8_010);
    assert!(json["iterations"][2]["output"]["claimed"].is_null());
    assert_eq!(queue.summary.completed, 2);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    Ok(())
}

#[tokio::test]
async fn worker_loop_with_hot_state_stops_at_poll_limit_without_empty_poll() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-run-scheduled").await?;
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-step-scheduled").await?;

    let output = must_ok(
        worker_loop_with_hot_state(
            &ledger,
            &hot_state,
            ActivityWorkerLoopStoreRequest {
                worker_id: "worker-loop",
                task_queue: None,
                now_ms: 8_000,
                now_step_ms: 1,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                poll_limit: 1,
                empty_limit: 1,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                settled_step_ms: 1,
                output_hash: Some("sha256:activity-output"),
                output_artifact_dir: None,
                output_artifact_kind: None,
                openai_compatible_base_url: None,
                openai_compatible_api_key: None,
                openai_compatible_timeout_ms: None,
                error_code: None,
                message: None,
                retryable: None,
                metadata: None,
                json: true,
            },
        )
        .await,
        "activity worker loop should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker loop output should be valid json",
    );

    assert_eq!(json["processed"], 1);
    assert_eq!(json["empty_polls"], 0);
    assert_eq!(json["stopped_reason"], "poll_limit");
    assert_eq!(json["iterations"].as_array().map(Vec::len), Some(1));
    Ok(())
}

#[tokio::test]
async fn worker_loop_with_hot_state_records_heartbeat_for_claimed_task() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-run-scheduled").await?;

    let output = must_ok(
        worker_loop_with_hot_state(
            &ledger,
            &hot_state,
            ActivityWorkerLoopStoreRequest {
                worker_id: "worker-loop",
                task_queue: None,
                now_ms: 8_000,
                now_step_ms: 1,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: Some(1_000),
                poll_limit: 1,
                empty_limit: 1,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                settled_step_ms: 1,
                output_hash: Some("sha256:activity-output"),
                output_artifact_dir: None,
                output_artifact_kind: None,
                openai_compatible_base_url: None,
                openai_compatible_api_key: None,
                openai_compatible_timeout_ms: None,
                error_code: None,
                message: None,
                retryable: None,
                metadata: None,
                json: true,
            },
        )
        .await,
        "activity worker loop should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker loop output should be valid json",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(8_500).await,
        "hot-state heartbeat snapshot should load",
    );
    let records = must_ok(
        ledger.load_events(&run_id),
        "worker loop heartbeat should persist an event",
    );
    let heartbeat_count = records
        .iter()
        .filter(|record| {
            matches!(
                &record.event.kind,
                ControlEventKind::WorkerHeartbeatObserved { .. }
            )
        })
        .count();

    assert_eq!(json["processed"], 1);
    assert_eq!(json["heartbeats"], 1);
    assert_eq!(
        json["iterations"][0]["output"]["heartbeat"]["event"]["kind"]["event"],
        "worker_heartbeat_observed"
    );
    assert_eq!(snapshot.live_heartbeat_count(), 1);
    assert_eq!(heartbeat_count, 1);
    Ok(())
}

#[tokio::test]
async fn worker_loop_with_hot_state_executes_openai_compatible_llm_to_artifact_dir()
-> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let prompt_path = temp_dir.path().join("prompt.txt");
    std::fs::write(&prompt_path, "Summarize durable workflow state.")
        .map_err(|error| format!("should write prompt fixture: {error}"))?;
    let output_dir = temp_dir.path().join("llm-output");
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"Durable summary."}}]}"#,
    )
    .await?;
    let run_id = append_control_run_with_openai_compatible_local_prompt(
        &ledger_path,
        &prompt_path,
        "activity-openai-compatible-loop",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(
        &ledger,
        &hot_state,
        &run_id,
        "activity-openai-compatible-loop",
    )
    .await?;

    let output = must_ok(
        worker_loop_with_hot_state(
            &ledger,
            &hot_state,
            ActivityWorkerLoopStoreRequest {
                worker_id: "worker-loop",
                task_queue: Some("llm.openrouter"),
                now_ms: 8_000,
                now_step_ms: 1,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                poll_limit: 2,
                empty_limit: 1,
                executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                settled_step_ms: 1,
                output_hash: None,
                output_artifact_dir: Some(output_dir.as_path()),
                output_artifact_kind: Some("llm.response"),
                openai_compatible_base_url: Some(base_url.as_str()),
                openai_compatible_api_key: Some("test-key"),
                openai_compatible_timeout_ms: Some(5_000),
                error_code: None,
                message: None,
                retryable: None,
                metadata: None,
                json: true,
            },
        )
        .await,
        "activity worker loop should execute OpenAI-compatible request",
    );
    let request = request_rx
        .await
        .map_err(|error| format!("provider request should be captured: {error}"))?;
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker loop output should be valid json",
    );
    let artifact_path =
        output_dir.join("activity-openai-compatible-loop-attempt-1.openai-compatible-llm.json");
    let artifact_json: serde_json::Value = must_ok(
        serde_json::from_str(
            &std::fs::read_to_string(&artifact_path)
                .map_err(|error| format!("should read provider artifact: {error}"))?,
        ),
        "provider artifact should be valid json",
    );

    assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(request.contains("authorization: Bearer test-key"));
    assert_eq!(json["processed"], 1);
    assert_eq!(json["released"], 1);
    assert_eq!(json["stopped_reason"], "empty_limit");
    assert_eq!(
        json["iterations"][0]["output"]["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]
            ["uri"],
        artifact_path.display().to_string()
    );
    assert_eq!(
        artifact_json["schema"],
        "qianji.openai_compatible_llm_response.v1"
    );
    assert_eq!(artifact_json["content"], "Durable summary.");
    Ok(())
}

async fn enqueue_worker_task(
    ledger: &DuckDbControlLedger,
    hot_state: &InMemoryHotStateStore,
    run_id: &xiuxian_qianji_control::RunId,
    activity_id: &str,
) -> Result<(), String> {
    let task = worker_task(ledger, run_id, activity_id)?;
    must_ok(
        hot_state
            .enqueue_activity_task(RunnableActivityTask {
                task,
                priority: 10,
                not_before_ms: 7_000,
                metadata: serde_json::json!({"mirror": "worker-loop"}),
            })
            .await,
        "should enqueue activity task",
    );
    Ok(())
}

fn worker_task(
    ledger: &DuckDbControlLedger,
    run_id: &xiuxian_qianji_control::RunId,
    activity_id: &str,
) -> Result<xiuxian_qianji_control::WorkerActivityTask, String> {
    must_ok(
        ledger.load_worker_activity_tasks(run_id, None),
        "should load worker activity tasks",
    )
    .into_iter()
    .find(|task| task.activity_id.as_str() == activity_id)
    .ok_or_else(|| format!("missing worker task for {activity_id}"))
}

fn append_control_run_with_openai_compatible_local_prompt(
    ledger_path: &std::path::Path,
    prompt_path: &std::path::Path,
    activity_id: &str,
) -> xiuxian_qianji_control::RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let activity_id = must_ok(
        ActivityId::new(activity_id),
        "should build governed LLM activity id",
    );
    let prompt_ref = ArtifactRef {
        artifact_id: must_ok(
            ArtifactId::new("artifact-openai-compatible-loop-prompt"),
            "should build prompt artifact id",
        ),
        artifact_kind: must_ok(
            ArtifactKind::new("llm.prompt"),
            "should build prompt artifact kind",
        ),
        uri: prompt_path.display().to_string(),
        content_digest: None,
        metadata: serde_json::Value::Null,
    };
    let mut task =
        activity_task(activity_id, "llm.plan", "llm.openrouter").with_input_ref(prompt_ref.clone());
    task.metadata = serde_json::json!({
        "qianji_llm_activity_request": {
            "schema": "qianji.llm_activity_request_audit.v1",
            "model": "openrouter/qwen/qwen3-coder",
            "prompt_ref": prompt_ref,
            "context_ref": null,
            "tool_schema_hash": null,
            "temperature_millis": 0,
            "max_tokens": 1024,
            "response_schema_ref": null,
            "budget": null,
            "request_metadata": null,
            "admission_metadata": null
        }
    });
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            2,
            ControlEventKind::ActivityScheduled { task },
        )),
        "should append governed OpenAI-compatible loop LLM route activity",
    );
    run_id
}

async fn spawn_openai_compatible_server(
    status: &'static str,
    body: &'static str,
) -> Result<(String, tokio::sync::oneshot::Receiver<String>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("should bind test provider server: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("should read test provider address: {error}"))?;
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

#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
#[test]
fn run_control_activity_worker_loop_requires_duckdb_and_valkey_features_without_connecting() {
    let temp_dir = must_ok(TempDir::new(), "should create temporary directory");
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);
    let error = must_some(
        run_control_command(&ControlCliCommand::ActivityWorkerLoop {
            ledger_path,
            valkey_url: "redis://127.0.0.1:1".to_string(),
            namespace: None,
            worker_id: "worker-loop".to_string(),
            task_queue: Some("llm.openai".to_string()),
            now_ms: 10,
            now_step_ms: 1,
            lease_ttl_ms: 50,
            heartbeat_ttl_ms: None,
            poll_limit: 1,
            empty_limit: 1,
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            settled_at_ms: 20,
            settled_step_ms: 1,
            output_hash: Some("sha256:activity-output".to_string()),
            output_artifact_dir: None,
            output_artifact_kind: None,
            openai_compatible_base_url: None,
            openai_compatible_api_key: None,
            openai_compatible_timeout_ms: None,
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
            json: true,
        })
        .err(),
        "activity worker loop should require duckdb and valkey features in partial builds",
    );

    assert!(
        error
            .to_string()
            .contains("`control activity-worker-loop` requires the `duckdb` and `valkey` features"),
        "unexpected error for run {run_id}: {error}"
    );
}
