use std::path::Path;

use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerLoopStoreRequest,
    worker_loop_with_hot_state,
};
use crate::qianji_cli::tests::control_cli::support::must_ok;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use xiuxian_qianji_control::{
    ControlEventKind, ControlLedger, HotStateStore, InMemoryControlLedger, InMemoryHotStateStore,
    RecoveryLoopApplicationRequest, RunId, apply_recovery_plan,
};

use super::support::{
    append_control_run_with_openai_compatible_local_prompt, enqueue_worker_task, recovery_attempt,
};

#[tokio::test]
async fn worker_loop_with_hot_state_executes_openai_compatible_llm_to_artifact_dir()
-> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let prompt_path = temp_dir.path().join("prompt.txt");
    std::fs::write(&prompt_path, "Summarize durable workflow state.")
        .map_err(|error| format!("should write prompt fixture: {error}"))?;
    let output_dir = temp_dir.path().join("llm-output");
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"Durable summary."}}]}"#,
    )
    .await?;
    let ledger = InMemoryControlLedger::new();
    let run_id = append_control_run_with_openai_compatible_local_prompt(
        &ledger,
        &prompt_path,
        "activity-openai-compatible-loop",
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
                worker_count: 1,
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

#[tokio::test]
async fn worker_loop_with_hot_state_records_openai_compatible_http_failure() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let prompt_path = temp_dir.path().join("prompt.txt");
    std::fs::write(&prompt_path, "Plan the next durable workflow repair.")
        .map_err(|error| format!("should write prompt fixture: {error}"))?;
    let output_dir = temp_dir.path().join("llm-output");
    let (base_url, request_rx) =
        spawn_openai_compatible_server("429 Too Many Requests", r#"{"error":"rate limited"}"#)
            .await?;
    let ledger = InMemoryControlLedger::new();
    let run_id = append_control_run_with_openai_compatible_local_prompt(
        &ledger,
        &prompt_path,
        "activity-openai-compatible-loop-http-failure",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(
        &ledger,
        &hot_state,
        &run_id,
        "activity-openai-compatible-loop-http-failure",
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
                worker_count: 1,
                executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                settled_step_ms: 1,
                output_hash: None,
                output_artifact_dir: Some(output_dir.as_path()),
                output_artifact_kind: Some("llm.response"),
                openai_compatible_base_url: Some(base_url.as_str()),
                openai_compatible_api_key: None,
                openai_compatible_timeout_ms: Some(5_000),
                error_code: None,
                message: None,
                retryable: None,
                metadata: None,
                json: true,
            },
        )
        .await,
        "activity worker loop should record OpenAI-compatible HTTP failure",
    );
    let _request = request_rx
        .await
        .map_err(|error| format!("provider request should be captured: {error}"))?;
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker loop output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker loop should replay OpenAI-compatible failure",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(9_100).await,
        "hot state snapshot should load after failed provider request",
    );
    let artifact_path = output_dir
        .join("activity-openai-compatible-loop-http-failure-attempt-1.openai-compatible-llm.json");

    assert!(!artifact_path.exists());
    assert_eq!(json["processed"], 1);
    assert_eq!(json["released"], 1);
    assert_eq!(json["stopped_reason"], "empty_limit");
    assert_eq!(
        json["iterations"][0]["output"]["terminal"]["record"]["event"]["kind"]["event"],
        "activity_failed"
    );
    assert_eq!(
        json["iterations"][0]["output"]["terminal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "provider_http_error"
    );
    assert_eq!(
        json["iterations"][0]["output"]["terminal"]["record"]["event"]["kind"]["failure"]["metadata"]
            ["http_status"],
        429
    );
    assert_eq!(queue.summary.failed, 1);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    Ok(())
}

#[tokio::test]
async fn worker_loop_with_hot_state_recovers_openai_compatible_failure() -> Result<(), String> {
    let proof = openai_compatible_loop_recovery_fixture().await?;
    let (failure_url, failure_request_rx) =
        spawn_openai_compatible_server("429 Too Many Requests", r#"{"error":"rate limited"}"#)
            .await?;
    let failure_json = run_openai_compatible_loop_pass(
        &proof.ledger,
        &proof.hot_state,
        proof.output_dir.as_path(),
        failure_url.as_str(),
        8_000,
        9_000,
        "first OpenAI-compatible loop pass should record durable failure",
    )
    .await?;
    let _failure_request = failure_request_rx
        .await
        .map_err(|error| format!("provider failure request should be captured: {error}"))?;

    assert_eq!(
        failure_json["iterations"][0]["output"]["terminal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "provider_http_error"
    );

    let recovery_plan = must_ok(
        proof.ledger.load_recovery_plan(&proof.run_id, 9_100),
        "failed provider activity should project a recovery plan",
    );
    assert_eq!(recovery_plan.summary().retry_activities, 1);
    let recovery = must_ok(
        apply_recovery_plan(
            &proof.ledger,
            &proof.hot_state,
            RecoveryLoopApplicationRequest::new(recovery_plan, recovery_attempt(), 9_100, 17),
        )
        .await,
        "bounded recovery loop should requeue the failed provider activity",
    );
    assert_eq!(recovery.action_results.len(), 1);
    let (success_url, success_request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"Recovered summary."}}]}"#,
    )
    .await?;
    let success_json = run_openai_compatible_loop_pass(
        &proof.ledger,
        &proof.hot_state,
        proof.output_dir.as_path(),
        success_url.as_str(),
        9_125,
        9_200,
        "second OpenAI-compatible loop pass should complete recovered task",
    )
    .await?;
    let _success_request = success_request_rx
        .await
        .map_err(|error| format!("provider success request should be captured: {error}"))?;
    let artifact_path = proof
        .output_dir
        .join("activity-openai-compatible-loop-recovery-attempt-2.openai-compatible-llm.json");
    let artifact_json = read_recovered_provider_artifact(artifact_path.as_path())?;
    let events = must_ok(
        proof.ledger.load_events(&proof.run_id),
        "recovered provider run should remain replayable",
    );
    let recovery_after_success = must_ok(
        proof.ledger.load_recovery_plan(&proof.run_id, 9_300),
        "completed recovered provider activity should not keep retry actions",
    );

    assert_eq!(
        success_json["iterations"][0]["output"]["terminal"]["record"]["event"]["kind"]["event"],
        "activity_completed"
    );
    assert_eq!(artifact_json["content"], "Recovered summary.");
    assert!(
        events.iter().any(|record| {
            matches!(&record.event.kind, ControlEventKind::RecoveryStarted { .. })
        })
    );
    assert!(events.iter().any(|record| {
        matches!(
            &record.event.kind,
            ControlEventKind::ActivityStarted {
                activity_id,
                attempt: 2,
                ..
            } if activity_id.as_str() == "activity-openai-compatible-loop-recovery"
        )
    }));
    assert_eq!(recovery_after_success.summary().retry_activities, 0);
    Ok(())
}

struct OpenAiCompatibleLoopRecoveryFixture {
    _temp_dir: TempDir,
    ledger: InMemoryControlLedger,
    hot_state: InMemoryHotStateStore,
    run_id: RunId,
    output_dir: std::path::PathBuf,
}

async fn openai_compatible_loop_recovery_fixture()
-> Result<OpenAiCompatibleLoopRecoveryFixture, String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let prompt_path = temp_dir.path().join("prompt.txt");
    std::fs::write(&prompt_path, "Recover the durable provider task.")
        .map_err(|error| format!("should write prompt fixture: {error}"))?;
    let output_dir = temp_dir.path().join("llm-output");
    let ledger = InMemoryControlLedger::new();
    let run_id = append_control_run_with_openai_compatible_local_prompt(
        &ledger,
        &prompt_path,
        "activity-openai-compatible-loop-recovery",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(
        &ledger,
        &hot_state,
        &run_id,
        "activity-openai-compatible-loop-recovery",
    )
    .await?;
    Ok(OpenAiCompatibleLoopRecoveryFixture {
        _temp_dir: temp_dir,
        ledger,
        hot_state,
        run_id,
        output_dir,
    })
}

async fn run_openai_compatible_loop_pass(
    ledger: &InMemoryControlLedger,
    hot_state: &InMemoryHotStateStore,
    output_dir: &Path,
    base_url: &str,
    now_ms: u64,
    settled_at_ms: u64,
    context: &str,
) -> Result<serde_json::Value, String> {
    let output = must_ok(
        worker_loop_with_hot_state(
            ledger,
            hot_state,
            ActivityWorkerLoopStoreRequest {
                worker_id: "worker-loop",
                task_queue: Some("llm.openrouter"),
                now_ms,
                now_step_ms: 1,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                poll_limit: 1,
                empty_limit: 1,
                worker_count: 1,
                executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms,
                settled_step_ms: 1,
                output_hash: None,
                output_artifact_dir: Some(output_dir),
                output_artifact_kind: Some("llm.response"),
                openai_compatible_base_url: Some(base_url),
                openai_compatible_api_key: None,
                openai_compatible_timeout_ms: Some(5_000),
                error_code: None,
                message: None,
                retryable: None,
                metadata: None,
                json: true,
            },
        )
        .await,
        context,
    );
    Ok(must_ok(
        serde_json::from_str(&output.rendered),
        "OpenAI-compatible loop output should be valid json",
    ))
}

fn read_recovered_provider_artifact(path: &Path) -> Result<serde_json::Value, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("should read recovered provider artifact: {error}"))?;
    Ok(must_ok(
        serde_json::from_str::<serde_json::Value>(&content),
        "recovered provider artifact should be valid json",
    ))
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
