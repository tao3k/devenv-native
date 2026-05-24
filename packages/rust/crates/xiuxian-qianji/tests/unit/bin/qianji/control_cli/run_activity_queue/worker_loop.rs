use std::path::Path;

use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerLoopStoreRequest,
    worker_loop_with_hot_state,
};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{activity_task, must_ok};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::tests::control_cli::support::{append_empty_control_run, must_some};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use xiuxian_qianji_control::{
    ActivityId, ActivityRetryPolicy, ArtifactId, ArtifactKind, ArtifactRef, ControlEvent,
    ControlEventKind, ControlLedger, HotStateStore, InMemoryControlLedger, InMemoryHotStateStore,
    RecoveryAttempt, RecoveryLoopApplicationRequest, RecoveryPolicy, RunId, RunnableActivityTask,
    StepId, apply_recovery_plan,
};

#[tokio::test]
async fn worker_loop_with_hot_state_processes_tasks_until_empty_limit() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let _temp_dir = temp_dir;
    let ledger = InMemoryControlLedger::new();
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger);
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
                worker_count: 1,
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
    assert_eq!(json["worker_count"], 1);
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
async fn worker_loop_with_hot_state_uses_bounded_worker_count() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let _temp_dir = temp_dir;
    let ledger = InMemoryControlLedger::new();
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger);
    append_scheduled_run_activity(
        &ledger,
        &run_id,
        7,
        "activity-run-scheduled-extra-1",
        "llm.openai",
    );
    append_scheduled_run_activity(
        &ledger,
        &run_id,
        8,
        "activity-run-scheduled-extra-2",
        "llm.openai",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-run-scheduled").await?;
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-step-scheduled").await?;
    enqueue_worker_task(
        &ledger,
        &hot_state,
        &run_id,
        "activity-run-scheduled-extra-1",
    )
    .await?;
    enqueue_worker_task(
        &ledger,
        &hot_state,
        &run_id,
        "activity-run-scheduled-extra-2",
    )
    .await?;

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
                poll_limit: 4,
                empty_limit: 1,
                worker_count: 2,
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
                metadata: Some("{\"rows\":4}"),
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

    assert_eq!(json["worker_count"], 2);
    assert_eq!(json["processed"], 4);
    assert_eq!(json["empty_polls"], 0);
    assert_eq!(json["released"], 4);
    assert_eq!(json["stopped_reason"], "poll_limit");
    assert_eq!(json["iterations"].as_array().map(Vec::len), Some(4));
    assert_worker_iteration_workers(
        &json,
        &[
            "worker-loop-1",
            "worker-loop-2",
            "worker-loop-1",
            "worker-loop-2",
        ],
    );
    assert_eq!(queue.summary.completed, 4);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    Ok(())
}

#[tokio::test]
async fn worker_loop_with_hot_state_stops_at_poll_limit_without_empty_poll() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let _temp_dir = temp_dir;
    let ledger = InMemoryControlLedger::new();
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger);
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
                worker_count: 1,
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
    let _temp_dir = temp_dir;
    let ledger = InMemoryControlLedger::new();
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger);
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
                worker_count: 1,
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

async fn enqueue_worker_task(
    ledger: &impl ControlLedger,
    hot_state: &InMemoryHotStateStore,
    run_id: &RunId,
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
    ledger: &impl ControlLedger,
    run_id: &RunId,
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
    ledger: &impl ControlLedger,
    prompt_path: &std::path::Path,
    activity_id: &str,
) -> RunId {
    let run_id = append_empty_control_run_to_ledger(ledger);
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
    task.retry_policy = Some(must_ok(
        ActivityRetryPolicy::new(3).map(|policy| policy.with_initial_interval_ms(25)),
        "should build governed LLM retry policy",
    ));
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

fn recovery_attempt() -> RecoveryAttempt {
    RecoveryAttempt {
        attempt: 1,
        reason: "recover OpenAI-compatible provider failure".to_string(),
        policy: RecoveryPolicy {
            max_attempts: 3,
            backoff_ms: 25,
            require_human_approval: false,
        },
    }
}

fn append_control_run_with_scheduled_activity_queue(ledger: &impl ControlLedger) -> RunId {
    let run_id = append_control_run_with_step(ledger);
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let scheduled_run_activity = must_ok(
        ActivityId::new("activity-run-scheduled"),
        "should build scheduled run activity id",
    );
    let started_activity = must_ok(
        ActivityId::new("activity-run-started"),
        "should build started run activity id",
    );
    let scheduled_step_activity = must_ok(
        ActivityId::new("activity-step-scheduled"),
        "should build scheduled step activity id",
    );

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            3,
            ControlEventKind::ActivityScheduled {
                task: activity_task(scheduled_run_activity, "llm.plan", "llm.openai"),
            },
        )),
        "should append scheduled run activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            4,
            ControlEventKind::ActivityScheduled {
                task: activity_task(started_activity.clone(), "llm.plan", "llm.openai"),
            },
        )),
        "should append started run activity schedule",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            5,
            ControlEventKind::ActivityStarted {
                activity_id: started_activity,
                worker_id: None,
                attempt: 1,
            },
        )),
        "should append started run activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            6,
            ControlEventKind::ActivityScheduled {
                task: activity_task(scheduled_step_activity, "tool.github", "tool.github"),
            },
        )),
        "should append scheduled step activity",
    );
    run_id
}

fn append_scheduled_run_activity(
    ledger: &impl ControlLedger,
    run_id: &RunId,
    sequence: u64,
    activity_id: &str,
    task_queue: &str,
) {
    let activity_id = must_ok(
        ActivityId::new(activity_id),
        "should build additional scheduled run activity id",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            sequence,
            ControlEventKind::ActivityScheduled {
                task: activity_task(activity_id, "llm.plan", task_queue),
            },
        )),
        "should append additional scheduled run activity",
    );
}

fn assert_worker_iteration_workers(json: &serde_json::Value, expected_worker_ids: &[&str]) {
    for (index, worker_id) in expected_worker_ids.iter().enumerate() {
        assert_eq!(json["iterations"][index]["output"]["worker_id"], *worker_id);
    }
}

fn append_control_run_with_step(ledger: &impl ControlLedger) -> RunId {
    let run_id = append_empty_control_run_to_ledger(ledger);
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            2,
            ControlEventKind::StepCreated {
                title: "Review durable state".to_string(),
                required_evidence: vec!["history_visible".to_string()],
                budget: None,
            },
        )),
        "should append step-created event",
    );
    run_id
}

fn append_empty_control_run_to_ledger(ledger: &impl ControlLedger) -> RunId {
    let run_id = must_ok(RunId::new("run-control-cli"), "should build control run id");
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            1,
            ControlEventKind::RunCreated {
                intent: "test qianji control recovery snapshot".to_string(),
                budget: None,
                metadata: serde_json::Value::Null,
            },
        )),
        "should append run-created event",
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
            worker_count: 1,
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
