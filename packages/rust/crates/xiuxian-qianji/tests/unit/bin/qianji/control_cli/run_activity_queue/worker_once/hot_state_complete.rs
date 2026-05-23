use std::path::Path;

use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerOnceStoreRequest,
    worker_once_with_hot_state,
};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, must_ok,
};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use xiuxian_qianji_control::{
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore,
};

use super::support::{
    append_control_run_with_activity_route,
    append_control_run_with_episteme_openai_compatible_local_prompt,
    append_control_run_with_llm_route, append_control_run_with_openai_compatible_local_prompt,
    enqueue_worker_task,
};

#[tokio::test]
async fn worker_once_with_hot_state_completes_and_releases_activity_lease() -> Result<(), String> {
    let output_ref_json = r#"{"artifact_id":"artifact-worker-output","artifact_kind":"llm.output","uri":"artifact://artifact-worker-output","content_digest":"sha256:activity-output","metadata":{"mime":"application/json"}}"#;
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
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &ActivityWorkerOnceStoreRequest {
                worker_id: "worker-once",
                task_queue: Some("llm.openai"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                output_ref_json: Some(output_ref_json),
                output_hash: Some("sha256:activity-output"),
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
                metadata: Some("{\"rows\":3}"),
                json: true,
            },
        )
        .await,
        "activity worker once complete should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once complete output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker once complete should replay into queue projection",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(9_100).await,
        "hot state snapshot should load after worker once complete",
    );

    assert_eq!(json["worker_id"], "worker-once");
    assert_eq!(json["task_queue"], "llm.openai");
    assert_eq!(json["executor"], "fixture");
    assert_fixture_executor_contract(&json);
    assert_eq!(json["outcome"], "complete");
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["activity_id"],
        "activity-run-scheduled"
    );
    assert_eq!(json["claimed"]["lease"]["worker_id"], "worker-once");
    assert_eq!(json["claimed"]["lease"]["expires_at_ms"], 8_500);
    assert_eq!(json["start"]["status"], "appended");
    assert_eq!(
        json["start"]["record"]["event"]["kind"]["event"],
        "activity_started"
    );
    assert_eq!(json["terminal"]["status"], "appended");
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["event"],
        "activity_completed"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_hash"],
        "sha256:activity-output"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["artifact_id"],
        "artifact-worker-output"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["artifact_kind"],
        "llm.output"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["uri"],
        "artifact://artifact-worker-output"
    );
    assert_eq!(json["released"], true);
    assert_eq!(queue.summary.completed, 1);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    Ok(())
}

fn assert_fixture_executor_contract(json: &serde_json::Value) {
    assert_eq!(json["executor_contract"]["executor"], "fixture");
    assert_eq!(json["executor_contract"]["adapter"], "fixture");
    assert_eq!(
        json["executor_contract"]["allowed_activity_types"],
        serde_json::json!([
            "llm.plan",
            "llm.tool_select",
            "llm.repair",
            "episteme.ontology.reasoning_fill",
            "tool.github",
            "wendao.search"
        ])
    );
    assert_eq!(
        json["executor_contract"]["allowed_task_queues"],
        serde_json::json!([
            "llm.openai",
            "llm.anthropic",
            "llm.openrouter",
            "llm.local",
            "episteme.ontology.reasoning",
            "tool.github",
            "wendao.search"
        ])
    );
    assert_eq!(json["executor_contract"]["requires_input_ref"], false);
}

#[tokio::test]
async fn worker_once_with_hot_state_writes_output_artifact_and_records_claim_check()
-> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let artifact_path = temp_dir.path().join("artifacts/llm-output.json");
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-run-scheduled").await?;

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &ActivityWorkerOnceStoreRequest {
                worker_id: "worker-once",
                task_queue: Some("llm.openai"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                output_ref_json: None,
                output_hash: None,
                output_artifact_path: Some(&artifact_path),
                output_artifact_dir: None,
                output_artifact_content: Some("{\"answer\":\"done\"}"),
                output_artifact_id: Some("artifact-worker-output"),
                output_artifact_kind: Some("llm.output"),
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
        "activity worker once should write a local output artifact",
    );
    let artifact_content = std::fs::read_to_string(&artifact_path)
        .map_err(|error| format!("should read output artifact: {error}"))?;
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once complete output should be valid json",
    );

    assert_eq!(artifact_content, "{\"answer\":\"done\"}");
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["artifact_id"],
        "artifact-worker-output"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["artifact_kind"],
        "llm.output"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["uri"],
        artifact_path.display().to_string()
    );
    assert!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_hash"]
            .as_str()
            .is_some_and(|hash| hash.starts_with("sha256:"))
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["content_digest"],
        json["terminal"]["record"]["event"]["kind"]["result"]["output_hash"]
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["metadata"]["source"],
        "qianji-control-activity-worker-once"
    );
    assert_eq!(json["released"], true);
    Ok(())
}

#[tokio::test]
async fn worker_once_with_hot_state_completes_episteme_reasoning_review_artifact()
-> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let artifact_path = temp_dir
        .path()
        .join("artifacts/episteme-reasoning-review.json");
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_activity_route(
        &ledger_path,
        "activity-episteme-reasoning-fill",
        "episteme.ontology.reasoning_fill",
        "episteme.ontology.reasoning",
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
        "activity-episteme-reasoning-fill",
    )
    .await?;

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &ActivityWorkerOnceStoreRequest {
                worker_id: "worker-episteme-fixture",
                task_queue: Some("episteme.ontology.reasoning"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                output_ref_json: None,
                output_hash: None,
                output_artifact_path: Some(&artifact_path),
                output_artifact_dir: None,
                output_artifact_content: Some(
                    r#"{"schema":"xiuxian.wendao.episteme.reasoning_fill_review_fixture.v1","status":"review_only","candidatePatchCount":0}"#,
                ),
                output_artifact_id: Some("artifact-episteme-reasoning-review"),
                output_artifact_kind: Some("episteme.reasoning_fill_review"),
                openai_compatible_base_url: None,
                openai_compatible_api_key: None,
                openai_compatible_timeout_ms: None,
                error_code: None,
                message: None,
                retryable: None,
                metadata: Some("{\"review_only\":true,\"rdf_mutation\":false}"),
                json: true,
            },
        )
        .await,
        "Episteme reasoning fixture should complete to a review artifact",
    );
    let artifact: serde_json::Value = must_ok(
        serde_json::from_str(
            &std::fs::read_to_string(&artifact_path)
                .map_err(|error| format!("should read review artifact: {error}"))?,
        ),
        "review artifact should be valid json",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker once should replay Episteme reasoning completion",
    );

    assert_eq!(
        artifact["schema"],
        "xiuxian.wendao.episteme.reasoning_fill_review_fixture.v1"
    );
    assert_eq!(artifact["status"], "review_only");
    assert_eq!(json["task_queue"], "episteme.ontology.reasoning");
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["activity_type"],
        "episteme.ontology.reasoning_fill"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["artifact_kind"],
        "episteme.reasoning_fill_review"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["metadata"]["review_only"],
        true
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["metadata"]["rdf_mutation"],
        false
    );
    assert_eq!(json["released"], true);
    assert_eq!(queue.summary.completed, 1);
    Ok(())
}

#[tokio::test]
async fn worker_once_with_hot_state_rejects_conflicting_output_artifact() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let artifact_path = temp_dir.path().join("artifacts/llm-output.json");
    std::fs::create_dir_all(
        artifact_path
            .parent()
            .ok_or_else(|| "artifact path should have parent".to_string())?,
    )
    .map_err(|error| format!("should create artifact directory: {error}"))?;
    std::fs::write(&artifact_path, "different")
        .map_err(|error| format!("should prewrite artifact: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-run-scheduled").await?;

    let error = worker_once_with_hot_state(
        &ledger,
        &hot_state,
        &ActivityWorkerOnceStoreRequest {
            worker_id: "worker-once",
            task_queue: Some("llm.openai"),
            now_ms: 8_000,
            lease_ttl_ms: 500,
            heartbeat_ttl_ms: None,
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            settled_at_ms: 9_000,
            output_ref_json: None,
            output_hash: None,
            output_artifact_path: Some(&artifact_path),
            output_artifact_dir: None,
            output_artifact_content: Some("{\"answer\":\"done\"}"),
            output_artifact_id: None,
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
    .await
    .err()
    .unwrap_or_else(|| panic!("conflicting output artifact should fail"));
    let records = must_ok(
        ledger.load_events(&run_id),
        "should load events after conflicting artifact failure",
    );

    assert!(
        error
            .to_string()
            .contains("already exists with different content"),
        "unexpected error: {error}"
    );
    assert!(
        records.iter().all(|record| !matches!(
            &record.event.kind,
            xiuxian_qianji_control::ControlEventKind::ActivityCompleted { .. }
        )),
        "conflicting artifact must not write durable ActivityCompleted"
    );
    Ok(())
}

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

#[tokio::test]
async fn worker_once_with_hot_state_executes_episteme_reasoning_openai_compatible_to_review_artifact()
-> Result<(), String> {
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"id":"chatcmpl-test","choices":[{"message":{"role":"assistant","content":"{\"schema\":\"xiuxian.wendao.episteme.reasoning_fill_review.v1\",\"status\":\"review_only\",\"candidatePatchCount\":0,\"rdfMutation\":false}"},"finish_reason":"stop"}]}"#,
    )
    .await?;
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let (ledger_path, prompt_path, context_path, output_path) =
        write_episteme_openai_compatible_fixture(temp_dir.path())?;
    let run_id = append_control_run_with_episteme_openai_compatible_local_prompt(
        &ledger_path,
        &prompt_path,
        &context_path,
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
        "activity-episteme-reasoning-openai-compatible-llm",
    )
    .await?;

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &episteme_openai_compatible_worker_request(base_url.as_str(), output_path.as_path()),
        )
        .await,
        "Episteme OpenAI-compatible worker once should execute and render",
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

    assert!(
        request.contains("Return an Episteme review artifact."),
        "provider request should include prompt text: {request}"
    );
    assert!(
        request.contains("fill.ltc.policy.001"),
        "provider request should include context text: {request}"
    );
    assert_eq!(
        artifact["schema"],
        "qianji.openai_compatible_llm_response.v1"
    );
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["activity_type"],
        "episteme.ontology.reasoning_fill"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["artifact_kind"],
        "episteme.reasoning_fill_review"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["metadata"]["executor"],
        "openai-compatible-llm"
    );
    assert_eq!(json["released"], true);
    Ok(())
}

fn write_episteme_openai_compatible_fixture(
    root: &Path,
) -> Result<
    (
        std::path::PathBuf,
        std::path::PathBuf,
        std::path::PathBuf,
        std::path::PathBuf,
    ),
    String,
> {
    let prompt_path = root.join("prompt.txt");
    let context_path = root.join("context.json");
    std::fs::write(&prompt_path, "Return an Episteme review artifact.")
        .map_err(|error| format!("should write prompt artifact: {error}"))?;
    std::fs::write(&context_path, r#"{"fillItemId":"fill.ltc.policy.001"}"#)
        .map_err(|error| format!("should write context artifact: {error}"))?;
    Ok((
        root.join("control.duckdb"),
        prompt_path,
        context_path,
        root.join("artifacts/episteme-reasoning-review.json"),
    ))
}

fn episteme_openai_compatible_worker_request<'a>(
    base_url: &'a str,
    output_path: &'a Path,
) -> ActivityWorkerOnceStoreRequest<'a> {
    ActivityWorkerOnceStoreRequest {
        worker_id: "worker-episteme-openrouter",
        task_queue: Some("episteme.ontology.reasoning"),
        now_ms: 8_000,
        lease_ttl_ms: 500,
        heartbeat_ttl_ms: None,
        executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
        outcome: ActivitySettleOutcomeArg::Complete,
        settled_at_ms: 9_000,
        output_ref_json: None,
        output_hash: None,
        output_artifact_path: Some(output_path),
        output_artifact_dir: None,
        output_artifact_content: None,
        output_artifact_id: Some("artifact-episteme-reasoning-review"),
        output_artifact_kind: Some("episteme.reasoning_fill_review"),
        openai_compatible_base_url: Some(base_url),
        openai_compatible_api_key: Some("test-key"),
        openai_compatible_timeout_ms: Some(5_000),
        error_code: None,
        message: None,
        retryable: None,
        metadata: None,
        json: true,
    }
}

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
        false
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
