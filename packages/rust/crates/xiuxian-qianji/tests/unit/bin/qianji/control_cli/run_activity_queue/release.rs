use crate::qianji_cli::test_exports::{WorkerActivityReleaseStoreRequest, release_with_hot_state};
use crate::qianji_cli::tests::control_cli::support::must_ok;
use xiuxian_qianji_control::{
    ActivityId, ActivityType, HotStateStore, IdempotencyKey, InMemoryHotStateStore, RunId,
    RunnableActivityTask, StepId, TaskQueue, WorkerActivityTask, WorkerId, WorkerRef,
};

#[tokio::test]
async fn release_with_hot_state_releases_claimed_activity_lease_json() -> Result<(), String> {
    let hot_state = InMemoryHotStateStore::new();
    let run_id = must_ok(RunId::new("run-release"), "should build run id");
    let step_id = must_ok(StepId::new("step-release"), "should build step id");
    let task_queue = must_ok(TaskQueue::new("llm.openai"), "should build task queue");
    let activity_id = must_ok(
        ActivityId::new("activity-release"),
        "should build activity id",
    );
    must_ok(
        hot_state
            .enqueue_activity_task(RunnableActivityTask {
                task: WorkerActivityTask {
                    run_id: run_id.clone(),
                    step_id: Some(step_id.clone()),
                    activity_id: activity_id.clone(),
                    activity_type: must_ok(ActivityType::new("llm.plan"), "activity type"),
                    task_queue: task_queue.clone(),
                    next_attempt: 1,
                    scheduled_at_ms: 10,
                    input_ref: None,
                    idempotency_key: must_ok(
                        IdempotencyKey::new("run-release/activity-release/1"),
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
    let claimed = must_ok(
        hot_state
            .claim_activity_task(
                WorkerRef {
                    worker_id: must_ok(WorkerId::new("worker-release"), "worker id"),
                    capabilities: Vec::new(),
                    metadata: serde_json::Value::Null,
                },
                Some(&task_queue),
                100,
                50,
            )
            .await,
        "should claim activity task",
    )
    .ok_or_else(|| "missing claimed activity task".to_string())?;
    let lease_json = must_ok(
        serde_json::to_string(&claimed.lease),
        "lease should serialize to json",
    );

    let output = must_ok(
        release_with_hot_state(
            &hot_state,
            WorkerActivityReleaseStoreRequest {
                lease_json: &lease_json,
                json: true,
            },
        )
        .await,
        "activity release should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "release output should be valid json",
    );

    assert_eq!(json["released"], true);
    assert_eq!(json["lease"]["lease_id"], claimed.lease.lease_id.as_str());
    assert_eq!(json["lease"]["activity_id"], activity_id.as_str());
    let snapshot = must_ok(
        hot_state.load_snapshot(110).await,
        "snapshot after release should load",
    );
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    Ok(())
}

#[tokio::test]
async fn release_with_hot_state_rejects_malformed_lease_json() -> Result<(), String> {
    let hot_state = InMemoryHotStateStore::new();
    let result = release_with_hot_state(
        &hot_state,
        WorkerActivityReleaseStoreRequest {
            lease_json: r#"{"lease_id":""}"#,
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
            .contains("invalid `--lease-json` for `control activity-release`"),
        "unexpected error: {error}"
    );
    Ok(())
}
