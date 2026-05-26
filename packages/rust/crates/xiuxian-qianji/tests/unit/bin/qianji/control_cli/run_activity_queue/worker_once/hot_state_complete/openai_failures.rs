use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerOnceStoreRequest,
    worker_once_with_hot_state,
};
use crate::qianji_cli::tests::control_cli::support::must_ok;
use tempfile::TempDir;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, InMemoryHotStateStore};

use super::support::spawn_openai_compatible_server;
use crate::qianji_cli::tests::control_cli::run_activity_queue::worker_once::support::{
    append_control_run_with_llm_route, append_control_run_with_openai_compatible_local_prompt,
    enqueue_worker_task,
};

#[tokio::test]
async fn worker_once_with_hot_state_records_openai_compatible_http_failure() -> Result<(), String> {
    let (base_url, request_rx) =
        spawn_openai_compatible_server("429 Too Many Requests", r#"{"error":"rate limit"}"#)
            .await?;
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let prompt_path = temp_dir.path().join("prompt.txt");
    std::fs::write(&prompt_path, "Plan the next durable step.")
        .map_err(|error| format!("should write prompt artifact: {error}"))?;
    let output_path = temp_dir.path().join("artifacts/llm-output.json");
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_openai_compatible_local_prompt(&ledger_path, &prompt_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(
        &ledger,
        &hot_state,
        &run_id,
        "activity-openai-compatible-llm",
    )
    .await?;

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &ActivityWorkerOnceStoreRequest {
                worker_id: "worker-openrouter",
                task_queue: Some("llm.openrouter"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                output_ref_json: None,
                output_hash: None,
                output_artifact_path: Some(&output_path),
                output_artifact_dir: None,
                output_artifact_content: None,
                output_artifact_id: Some("artifact-openai-compatible-output"),
                output_artifact_kind: Some("llm.output"),
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
        "OpenAI-compatible HTTP failure should settle as durable failure",
    );
    let _request = request_rx
        .await
        .map_err(|error| format!("should receive provider request: {error}"))?;
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker once should replay OpenAI-compatible failure",
    );

    assert!(!output_path.exists());
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["event"],
        "activity_failed"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "provider_http_error"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["metadata"]["http_status"],
        429
    );
    assert_eq!(json["released"], true);
    assert_eq!(queue.summary.failed, 1);
    Ok(())
}

#[tokio::test]
async fn worker_once_with_hot_state_records_openai_compatible_malformed_response()
-> Result<(), String> {
    let (base_url, request_rx) = spawn_openai_compatible_server("200 OK", "not-json").await?;
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let prompt_path = temp_dir.path().join("prompt.txt");
    std::fs::write(&prompt_path, "Plan the next durable step.")
        .map_err(|error| format!("should write prompt artifact: {error}"))?;
    let output_path = temp_dir.path().join("artifacts/llm-output.json");
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_openai_compatible_local_prompt(&ledger_path, &prompt_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(
        &ledger,
        &hot_state,
        &run_id,
        "activity-openai-compatible-llm",
    )
    .await?;

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &ActivityWorkerOnceStoreRequest {
                worker_id: "worker-openrouter",
                task_queue: Some("llm.openrouter"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                output_ref_json: None,
                output_hash: None,
                output_artifact_path: Some(&output_path),
                output_artifact_dir: None,
                output_artifact_content: None,
                output_artifact_id: Some("artifact-openai-compatible-output"),
                output_artifact_kind: Some("llm.output"),
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
        "OpenAI-compatible malformed response should settle as durable failure",
    );
    let _request = request_rx
        .await
        .map_err(|error| format!("should receive provider request: {error}"))?;
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker once should replay OpenAI-compatible malformed response",
    );

    assert!(!output_path.exists());
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["event"],
        "activity_failed"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "provider_malformed_response"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["retryable"],
        true
    );
    assert_eq!(json["released"], true);
    assert_eq!(queue.summary.failed, 1);
    Ok(())
}

#[tokio::test]
async fn worker_once_with_hot_state_accepts_llm_repair_openrouter_route() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_llm_route(
        &ledger_path,
        "activity-llm-repair",
        "llm.repair",
        "llm.openrouter",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-llm-repair").await?;

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &ActivityWorkerOnceStoreRequest {
                worker_id: "worker-openrouter",
                task_queue: Some("llm.openrouter"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                output_ref_json: None,
                output_hash: Some("sha256:llm-repair-output"),
                output_artifact_path: None,
                output_artifact_dir: None,
                output_artifact_content: None,
                output_artifact_id: None,
                output_artifact_kind: None,
                openai_compatible_base_url: None,
                openai_compatible_api_key: None,
                openai_compatible_timeout_ms: None,
                error_code: None,
                message: None,
                retryable: None,
                metadata: Some("{\"provider\":\"openrouter\"}"),
                json: true,
            },
        )
        .await,
        "activity worker once should accept governed LLM repair route",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );

    assert_eq!(json["task_queue"], "llm.openrouter");
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["activity_type"],
        "llm.repair"
    );
    assert_eq!(json["start"]["status"], "appended");
    assert_eq!(json["terminal"]["status"], "appended");
    assert_eq!(json["released"], true);
    Ok(())
}
