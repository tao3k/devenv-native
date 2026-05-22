use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerLoopStoreRequest,
    worker_loop_with_hot_state,
};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, must_ok,
};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::tests::control_cli::support::{append_empty_control_run, must_some};
use tempfile::TempDir;
use xiuxian_qianji_control::{
    ControlEventKind, ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore,
    RunnableActivityTask,
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
