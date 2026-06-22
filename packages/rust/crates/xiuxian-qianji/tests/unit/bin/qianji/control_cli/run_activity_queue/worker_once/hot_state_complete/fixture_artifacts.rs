use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerOnceStoreRequest,
    worker_once_with_hot_state,
};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, must_ok,
};
use tempfile::TempDir;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, InMemoryHotStateStore};

use crate::qianji_cli::tests::control_cli::run_activity_queue::worker_once::support::{
    append_control_run_with_activity_route, enqueue_worker_task,
};

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
