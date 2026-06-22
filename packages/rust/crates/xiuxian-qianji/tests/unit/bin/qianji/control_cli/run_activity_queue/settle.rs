use crate::qianji_cli::test_exports::{
    ActivitySettleOutcomeArg, WorkerActivitySettleStoreRequest, WorkerActivityTakeStoreRequest,
    settle_with_hot_state, take_with_hot_state,
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
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore, RunnableActivityTask,
    WorkerId, WorkerRef,
};

#[tokio::test]
async fn settle_with_hot_state_completes_then_releases_activity_lease() -> Result<(), String> {
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
    let leased_task_json = take_leased_task_json(&ledger, &hot_state).await?;

    let output = must_ok(
        settle_with_hot_state(
            &ledger,
            &hot_state,
            WorkerActivitySettleStoreRequest {
                leased_task_json: &leased_task_json,
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
        "activity settle complete should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity settle complete output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "settle complete should replay into queue projection",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(9_100).await,
        "hot state snapshot should load after settle",
    );

    assert_eq!(json["outcome"], "complete");
    assert_eq!(json["journal"]["status"], "appended");
    assert_eq!(
        json["journal"]["record"]["event"]["kind"]["event"],
        "activity_completed"
    );
    assert_eq!(
        json["journal"]["record"]["event"]["kind"]["result"]["output_hash"],
        "sha256:activity-output"
    );
    assert_eq!(json["released"], true);
    assert_eq!(queue.summary.completed, 1);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    Ok(())
}

#[tokio::test]
async fn settle_with_hot_state_fails_then_releases_activity_lease() -> Result<(), String> {
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
    let leased_task_json = take_leased_task_json(&ledger, &hot_state).await?;

    let output = must_ok(
        settle_with_hot_state(
            &ledger,
            &hot_state,
            WorkerActivitySettleStoreRequest {
                leased_task_json: &leased_task_json,
                outcome: ActivitySettleOutcomeArg::Fail,
                settled_at_ms: 9_000,
                output_hash: None,
                error_code: Some("rate_limited"),
                message: Some("provider rejected request"),
                retryable: Some(true),
                metadata: None,
                json: true,
            },
        )
        .await,
        "activity settle fail should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity settle fail output should be valid json",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "settle fail should replay into queue projection",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(9_100).await,
        "hot state snapshot should load after settle failure",
    );

    assert_eq!(json["outcome"], "fail");
    assert_eq!(json["journal"]["status"], "appended");
    assert_eq!(
        json["journal"]["record"]["event"]["kind"]["event"],
        "activity_failed"
    );
    assert_eq!(
        json["journal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "rate_limited"
    );
    assert_eq!(json["released"], true);
    assert_eq!(queue.summary.failed, 1);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    Ok(())
}

#[tokio::test]
async fn settle_with_hot_state_keeps_lease_when_durable_write_fails() -> Result<(), String> {
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
    let worker_task = worker_task(&ledger, &run_id, "activity-run-scheduled")?;
    let claimed = must_ok(
        hot_state
            .claim_activity_task(
                WorkerRef {
                    worker_id: must_ok(WorkerId::new("worker-settle"), "worker id"),
                    capabilities: Vec::new(),
                    metadata: serde_json::Value::Null,
                },
                Some(&worker_task.task_queue),
                8_000,
                500,
            )
            .await,
        "should claim activity task without durable start",
    )
    .ok_or_else(|| "missing claimed activity task".to_string())?;
    let leased_task_json = must_ok(
        serde_json::to_string(&claimed),
        "claimed activity task should serialize",
    );

    let result = settle_with_hot_state(
        &ledger,
        &hot_state,
        WorkerActivitySettleStoreRequest {
            leased_task_json: &leased_task_json,
            outcome: ActivitySettleOutcomeArg::Complete,
            settled_at_ms: 9_000,
            output_hash: None,
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
            json: true,
        },
    )
    .await;
    let error = match result {
        Ok(output) => {
            return Err(format!(
                "settle before durable start should fail, rendered: {}",
                output.rendered
            ));
        }
        Err(error) => error,
    };
    let snapshot = must_ok(
        hot_state.load_snapshot(8_100).await,
        "hot state snapshot should load after failed settle",
    );

    assert!(
        error
            .to_string()
            .contains("activity completion requires a started activity"),
        "unexpected error: {error}"
    );
    assert_eq!(snapshot.active_activity_task_lease_count(), 1);
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
                metadata: serde_json::json!({"mirror": "settle"}),
            })
            .await,
        "should enqueue activity task",
    );
    Ok(())
}

async fn take_leased_task_json(
    ledger: &DuckDbControlLedger,
    hot_state: &InMemoryHotStateStore,
) -> Result<String, String> {
    let output = must_ok(
        take_with_hot_state(
            ledger,
            hot_state,
            WorkerActivityTakeStoreRequest {
                worker_id: "worker-settle",
                task_queue: None,
                now_ms: 8_000,
                lease_ttl_ms: 500,
                json: true,
            },
        )
        .await,
        "activity take should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity take output should be valid json",
    );
    serde_json::to_string(&json["claimed"])
        .map_err(|error| format!("claimed task should serialize: {error}"))
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
fn run_control_activity_settle_requires_duckdb_and_valkey_features_without_connecting() {
    let temp_dir = must_ok(TempDir::new(), "should create temporary directory");
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);
    let error = must_some(
        run_control_command(&ControlCliCommand::ActivitySettle {
            ledger_path,
            valkey_url: "redis://127.0.0.1:1".to_string(),
            namespace: None,
            leased_task_json: "{}".to_string(),
            outcome: ActivitySettleOutcomeArg::Complete,
            settled_at_ms: 10,
            output_hash: None,
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
            json: true,
        })
        .err(),
        "activity settle should require duckdb and valkey features in partial builds",
    );

    assert!(
        error
            .to_string()
            .contains("`control activity-settle` requires the `duckdb` and `valkey` features"),
        "unexpected error for run {run_id}: {error}"
    );
}
