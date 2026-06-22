use crate::qianji_cli::test_exports::{WorkerActivityReclaimStoreRequest, reclaim_with_hot_state};
use crate::qianji_cli::tests::control_cli::support::must_ok;
use xiuxian_qianji_control::{
    ActivityId, ActivityType, HotStateStore, IdempotencyKey, InMemoryHotStateStore, RunId,
    RunnableActivityTask, StepId, TaskQueue, WorkerActivityTask, WorkerId, WorkerRef,
};

#[tokio::test]
async fn reclaim_with_hot_state_requeues_expired_activity_lease_json() -> Result<(), String> {
    let hot_state = InMemoryHotStateStore::new();
    let run_id = must_ok(RunId::new("run-reclaim"), "should build run id");
    let step_id = must_ok(StepId::new("step-reclaim"), "should build step id");
    let task_queue = must_ok(TaskQueue::new("tool.github"), "should build task queue");
    let activity_id = must_ok(
        ActivityId::new("activity-reclaim"),
        "should build activity id",
    );
    must_ok(
        hot_state
            .enqueue_activity_task(RunnableActivityTask {
                task: WorkerActivityTask {
                    run_id: run_id.clone(),
                    step_id: Some(step_id.clone()),
                    activity_id: activity_id.clone(),
                    activity_type: must_ok(ActivityType::new("tool.github"), "activity type"),
                    task_queue: task_queue.clone(),
                    next_attempt: 1,
                    scheduled_at_ms: 10,
                    input_ref: None,
                    idempotency_key: must_ok(
                        IdempotencyKey::new("run-reclaim/activity-reclaim/1"),
                        "idempotency key",
                    ),
                    retry_policy: None,
                    timeout_ms: Some(30_000),
                    metadata: serde_json::Value::Null,
                },
                priority: 7,
                not_before_ms: 100,
                metadata: serde_json::json!({"mirror": "unit"}),
            })
            .await,
        "should enqueue activity task",
    );
    let worker = WorkerRef {
        worker_id: must_ok(WorkerId::new("worker-reclaim"), "worker id"),
        capabilities: Vec::new(),
        metadata: serde_json::Value::Null,
    };
    let claimed = must_ok(
        hot_state
            .claim_activity_task(worker.clone(), Some(&task_queue), 100, 50)
            .await,
        "should claim activity task",
    )
    .ok_or_else(|| "missing claimed activity task".to_string())?;
    let lease_json = must_ok(
        serde_json::to_string(&claimed.lease),
        "lease should serialize to json",
    );

    let active_output = must_ok(
        reclaim_with_hot_state(
            &hot_state,
            WorkerActivityReclaimStoreRequest {
                lease_json: &lease_json,
                now_ms: 149,
                json: true,
            },
        )
        .await,
        "active activity reclaim should render",
    );
    let active_json: serde_json::Value = must_ok(
        serde_json::from_str(&active_output.rendered),
        "active reclaim output should be valid json",
    );
    assert_eq!(active_json["reclaimed"], false);

    let expired_output = must_ok(
        reclaim_with_hot_state(
            &hot_state,
            WorkerActivityReclaimStoreRequest {
                lease_json: &lease_json,
                now_ms: 150,
                json: true,
            },
        )
        .await,
        "expired activity reclaim should render",
    );
    let expired_json: serde_json::Value = must_ok(
        serde_json::from_str(&expired_output.rendered),
        "expired reclaim output should be valid json",
    );
    assert_eq!(expired_json["reclaimed"], true);

    let requeued = must_ok(
        hot_state
            .claim_activity_task(worker, Some(&task_queue), 151, 50)
            .await,
        "should claim reclaimed activity task",
    )
    .ok_or_else(|| "missing reclaimed activity task".to_string())?;
    assert_eq!(requeued.activity_task.task.activity_id, activity_id);
    assert_ne!(requeued.lease.lease_id, claimed.lease.lease_id);
    Ok(())
}

#[tokio::test]
async fn reclaim_with_hot_state_rejects_malformed_lease_json() -> Result<(), String> {
    let hot_state = InMemoryHotStateStore::new();
    let result = reclaim_with_hot_state(
        &hot_state,
        WorkerActivityReclaimStoreRequest {
            lease_json: r#"{"lease_id":""}"#,
            now_ms: 150,
            json: true,
        },
    )
    .await;
    let error = match result {
        Ok(output) => {
            return Err(format!(
                "malformed lease json should fail, rendered: {}",
                output.rendered
            ));
        }
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("invalid `--lease-json` for `control activity-reclaim`"),
        "unexpected error: {error}"
    );
    Ok(())
}
