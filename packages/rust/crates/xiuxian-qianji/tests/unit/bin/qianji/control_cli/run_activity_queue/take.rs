#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::test_exports::{WorkerActivityTakeStoreRequest, take_with_hot_state};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, must_ok,
};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::tests::control_cli::support::{append_empty_control_run, must_some};
use tempfile::TempDir;
use xiuxian_qianji_control::{
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore, RunnableActivityTask,
};

#[tokio::test]
async fn take_with_hot_state_claims_task_and_records_durable_start() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    let worker_task = worker_task(&ledger, &run_id, "activity-run-scheduled")?;
    must_ok(
        hot_state
            .enqueue_activity_task(RunnableActivityTask {
                task: worker_task,
                priority: 10,
                not_before_ms: 7_000,
                metadata: serde_json::json!({"mirror": "take"}),
            })
            .await,
        "should enqueue activity task",
    );

    let output = must_ok(
        take_with_hot_state(
            &ledger,
            &hot_state,
            WorkerActivityTakeStoreRequest {
                worker_id: "worker-take",
                task_queue: Some("llm.openai"),
                now_ms: 7_000,
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
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "activity take should replay into queue projection",
    );

    assert_eq!(json["worker_id"], "worker-take");
    assert_eq!(json["task_queue"], "llm.openai");
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["activity_id"],
        "activity-run-scheduled"
    );
    assert_eq!(json["claimed"]["lease"]["worker_id"], "worker-take");
    assert_eq!(json["claimed"]["lease"]["expires_at_ms"], 7_500);
    assert_eq!(json["start"]["status"], "appended");
    assert_eq!(
        json["start"]["record"]["event"]["kind"]["event"],
        "activity_started"
    );
    assert_eq!(
        json["start"]["record"]["event"]["kind"]["worker_id"],
        "worker-take"
    );
    assert_eq!(queue.summary.total, 3);
    assert_eq!(queue.summary.scheduled, 1);
    assert_eq!(queue.summary.in_flight, 2);
    Ok(())
}

#[tokio::test]
async fn take_with_hot_state_does_not_write_ledger_when_queue_is_empty() -> Result<(), String> {
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
        "should load events before empty take",
    )
    .len();

    let output = must_ok(
        take_with_hot_state(
            &ledger,
            &hot_state,
            WorkerActivityTakeStoreRequest {
                worker_id: "worker-empty",
                task_queue: Some("llm.openai"),
                now_ms: 7_000,
                lease_ttl_ms: 500,
                json: true,
            },
        )
        .await,
        "empty activity take should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "empty activity take output should be valid json",
    );
    let after = must_ok(
        ledger.load_events(&run_id),
        "should load events after empty take",
    )
    .len();

    assert_eq!(json["worker_id"], "worker-empty");
    assert!(json["claimed"].is_null());
    assert!(json["start"].is_null());
    assert_eq!(before, after);
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
fn run_control_activity_take_requires_duckdb_and_valkey_features_without_connecting() {
    let temp_dir = must_ok(TempDir::new(), "should create temporary directory");
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);
    let error = must_some(
        run_control_command(&ControlCliCommand::ActivityTake {
            ledger_path,
            valkey_url: "redis://127.0.0.1:1".to_string(),
            namespace: None,
            worker_id: "worker-take".to_string(),
            task_queue: Some("llm.openai".to_string()),
            now_ms: 10,
            lease_ttl_ms: 50,
            json: true,
        })
        .err(),
        "activity take should require duckdb and valkey features in partial builds",
    );

    assert!(
        error
            .to_string()
            .contains("`control activity-take` requires the `duckdb` and `valkey` features"),
        "unexpected error for run {run_id}: {error}"
    );
}
