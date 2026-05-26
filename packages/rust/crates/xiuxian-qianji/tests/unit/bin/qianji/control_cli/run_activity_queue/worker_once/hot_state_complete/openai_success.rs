use std::path::Path;

use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerOnceStoreRequest,
    worker_once_with_hot_state,
};
use crate::qianji_cli::tests::control_cli::support::must_ok;
use tempfile::TempDir;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, InMemoryHotStateStore};

use super::support::spawn_openai_compatible_server;
use crate::qianji_cli::tests::control_cli::run_activity_queue::worker_once::support::{
    append_control_run_with_openai_compatible_local_prompt, enqueue_worker_task,
};

#[tokio::test]
async fn worker_once_with_hot_state_executes_openai_compatible_llm_to_artifact()
-> Result<(), String> {
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"id":"chatcmpl-test","choices":[{"message":{"role":"assistant","content":"planned answer"},"finish_reason":"stop"}]}"#,
    )
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
                openai_compatible_api_key: Some("\"test-key\""),
                openai_compatible_timeout_ms: Some(5_000),
                error_code: None,
                message: None,
                retryable: None,
                metadata: None,
                json: true,
            },
        )
        .await,
        "OpenAI-compatible worker once should execute and render",
    );
    let request = request_rx
        .await
        .map_err(|error| format!("should receive provider request: {error}"))?;
    let artifact: serde_json::Value = must_ok(
        serde_json::from_str(
            &std::fs::read_to_string(&output_path)
                .map_err(|error| format!("should read provider artifact: {error}"))?,
        ),
        "provider artifact should be valid json",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker once should replay OpenAI-compatible completion",
    );

    assert_openai_compatible_request(&request);
    assert_openai_compatible_artifact(&artifact);
    assert_openai_compatible_worker_output(&json, output_path.as_path());
    assert_eq!(json["released"], true);
    assert_eq!(queue.summary.completed, 1);
    Ok(())
}

fn assert_openai_compatible_request(request: &str) {
    assert!(
        request
            .to_ascii_lowercase()
            .contains("authorization: bearer test-key"),
        "provider request should include bearer auth: {request}"
    );
    assert!(
        request.contains("Plan the next durable step."),
        "provider request should include prompt text: {request}"
    );
    assert!(
        request.contains("openrouter/qwen/qwen3-coder"),
        "provider request should include admitted model: {request}"
    );
}

fn assert_openai_compatible_artifact(artifact: &serde_json::Value) {
    assert_eq!(
        artifact["schema"],
        "qianji.openai_compatible_llm_response.v1"
    );
    assert_eq!(artifact["content"], "planned answer");
}

fn assert_openai_compatible_worker_output(json: &serde_json::Value, output_path: &Path) {
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["event"],
        "activity_completed"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["artifact_id"],
        "artifact-openai-compatible-output"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["uri"],
        output_path.display().to_string()
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["metadata"]["executor"],
        "openai-compatible-llm"
    );
}
