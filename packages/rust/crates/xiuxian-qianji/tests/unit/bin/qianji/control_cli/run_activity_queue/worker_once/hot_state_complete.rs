use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerOnceStoreRequest,
    worker_once_with_hot_state,
};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, must_ok,
};
use tempfile::TempDir;
use xiuxian_qianji_control::{
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore,
};

use super::support::{append_control_run_with_llm_route, enqueue_worker_task};

#[tokio::test]
async fn worker_once_with_hot_state_completes_and_releases_activity_lease() -> Result<(), String> {
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
            ActivityWorkerOnceStoreRequest {
                worker_id: "worker-once",
                task_queue: Some("llm.openai"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                output_hash: Some("sha256:activity-output"),
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
    assert_eq!(json["executor_contract"]["executor"], "fixture");
    assert_eq!(json["executor_contract"]["adapter"], "fixture");
    assert_eq!(
        json["executor_contract"]["allowed_activity_types"],
        serde_json::json!([
            "llm.plan",
            "llm.tool_select",
            "llm.repair",
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
            "tool.github",
            "wendao.search"
        ])
    );
    assert_eq!(json["executor_contract"]["requires_input_ref"], false);
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
    assert_eq!(json["released"], true);
    assert_eq!(queue.summary.completed, 1);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
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
            ActivityWorkerOnceStoreRequest {
                worker_id: "worker-openrouter",
                task_queue: Some("llm.openrouter"),
                now_ms: 8_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 9_000,
                output_hash: Some("sha256:llm-repair-output"),
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
