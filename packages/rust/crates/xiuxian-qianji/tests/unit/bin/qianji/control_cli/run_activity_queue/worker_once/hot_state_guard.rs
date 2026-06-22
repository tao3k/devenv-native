use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerOnceStoreRequest,
    worker_once_with_hot_state,
};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, must_ok,
};
use tempfile::TempDir;
use xiuxian_qianji_control::{
    ControlEventKind, ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore,
};

use super::support::{
    append_control_run_with_disallowed_activity_route,
    append_control_run_with_openai_compatible_llm_route, enqueue_worker_task,
};

#[tokio::test]
async fn worker_once_with_hot_state_fails_and_releases_activity_lease() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-step-scheduled").await?;

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &ActivityWorkerOnceStoreRequest {
                worker_id: "worker-once",
                task_queue: Some("tool.github"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Fail,
                settled_at_ms: 9_000,
                output_ref_json: None,
                output_hash: None,
                output_artifact_path: None,
                output_artifact_dir: None,
                output_artifact_content: None,
                output_artifact_id: None,
                output_artifact_kind: None,
                openai_compatible_base_url: None,
                openai_compatible_api_key: None,
                openai_compatible_timeout_ms: None,
                error_code: Some("rate_limited"),
                message: Some("provider rejected request"),
                retryable: Some(true),
                metadata: Some("{\"provider\":\"openrouter\"}"),
                json: true,
            },
        )
        .await,
        "activity worker once fail should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once fail output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker once fail should replay into queue projection",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(9_100).await,
        "hot state snapshot should load after worker once fail",
    );

    assert_eq!(json["outcome"], "fail");
    assert_eq!(json["terminal"]["status"], "appended");
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["event"],
        "activity_failed"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "rate_limited"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["metadata"]["provider"],
        "openrouter"
    );
    assert_eq!(json["released"], true);
    assert_eq!(queue.summary.failed, 1);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    Ok(())
}

#[tokio::test]
async fn worker_once_with_hot_state_does_not_write_ledger_when_queue_is_empty() -> Result<(), String>
{
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    let before = must_ok(
        ledger.load_events(&run_id),
        "should load events before empty worker once",
    )
    .len();

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &ActivityWorkerOnceStoreRequest {
                worker_id: "worker-empty",
                task_queue: Some("llm.openai"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                output_ref_json: None,
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
                metadata: None,
                json: true,
            },
        )
        .await,
        "empty activity worker once should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "empty activity worker once output should be valid json",
    );
    let after = must_ok(
        ledger.load_events(&run_id),
        "should load events after empty worker once",
    )
    .len();

    assert_eq!(json["worker_id"], "worker-empty");
    assert!(json["executor_contract"].is_null());
    assert!(json["claimed"].is_null());
    assert!(json["start"].is_null());
    assert!(json["terminal"].is_null());
    assert_eq!(json["released"], false);
    assert_eq!(before, after);
    Ok(())
}

#[tokio::test]
async fn worker_once_rejects_disallowed_route_before_durable_start() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_disallowed_activity_route(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(&ledger, &hot_state, &run_id, "activity-disallowed-route").await?;

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
            metadata: None,
            json: true,
        },
    )
    .await
    .err()
    .unwrap_or_else(|| panic!("disallowed executor route should fail before durable start"));
    let records = must_ok(
        ledger.load_events(&run_id),
        "should load events after disallowed route",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(8_100).await,
        "hot state snapshot should retain active lease for reclaim",
    );

    assert!(
        error.to_string().contains(
            "activity executor `Fixture` does not allow activity_type `provider.unknown`"
        ),
        "unexpected error: {error}"
    );
    assert!(
        records
            .iter()
            .all(|record| !matches!(&record.event.kind, ControlEventKind::ActivityStarted { .. })),
        "disallowed route must not write durable ActivityStarted"
    );
    assert_eq!(snapshot.active_activity_task_lease_count(), 1);
    Ok(())
}

#[tokio::test]
async fn worker_once_rejects_openai_compatible_gate_before_durable_start() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_openai_compatible_llm_route(&ledger_path);
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

    let error = worker_once_with_hot_state(
        &ledger,
        &hot_state,
        &ActivityWorkerOnceStoreRequest {
            worker_id: "worker-once",
            task_queue: Some("llm.openrouter"),
            now_ms: 8_000,
            lease_ttl_ms: 500,
            heartbeat_ttl_ms: None,
            executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
            outcome: ActivitySettleOutcomeArg::Complete,
            settled_at_ms: 9_000,
            output_ref_json: None,
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
            metadata: None,
            json: true,
        },
    )
    .await
    .err()
    .unwrap_or_else(|| panic!("OpenAI-compatible gate should stop before durable start"));
    let records = must_ok(
        ledger.load_events(&run_id),
        "should load events after OpenAI-compatible gate",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(8_100).await,
        "hot state snapshot should retain active lease for reclaim",
    );

    let expected_error = "local Qianji LLM provider execution is retired";
    assert!(
        error.to_string().contains(expected_error),
        "unexpected error: {error}"
    );
    assert!(
        records
            .iter()
            .all(|record| !matches!(&record.event.kind, ControlEventKind::ActivityStarted { .. })),
        "admission-only executor must not write durable ActivityStarted"
    );
    assert_eq!(snapshot.active_activity_task_lease_count(), 1);
    Ok(())
}
