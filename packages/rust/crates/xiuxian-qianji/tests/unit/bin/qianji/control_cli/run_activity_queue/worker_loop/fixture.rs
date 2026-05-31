use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerLoopStoreRequest,
    worker_loop_with_hot_state,
};
use crate::qianji_cli::tests::control_cli::support::must_ok;
use tempfile::TempDir;
use xiuxian_qianji_control::{
    ControlEventKind, ControlLedger, HotStateStore, InMemoryControlLedger, InMemoryHotStateStore,
};

use super::support::{
    append_control_run_with_scheduled_activity_queue, append_scheduled_run_activity,
    assert_worker_iteration_workers, enqueue_worker_task,
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
    assert_eq!(
        json["iterations"][0]["output"]["heartbeat"]["event"]["kind"]["heartbeat"]["metadata"]["executor"],
        "fixture"
    );
    assert_eq!(
        json["iterations"][0]["output"]["heartbeat"]["event"]["kind"]["heartbeat"]["metadata"]["phase"],
        "activity_execution_active"
    );
    assert_eq!(snapshot.live_heartbeat_count(), 1);
    assert_eq!(heartbeat_count, 1);
    Ok(())
}
