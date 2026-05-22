#[cfg(not(feature = "valkey"))]
use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::test_exports::{WorkerActivityClaimStoreRequest, claim_with_hot_state};
use crate::qianji_cli::tests::control_cli::support::must_ok;
#[cfg(not(feature = "valkey"))]
use crate::qianji_cli::tests::control_cli::support::must_some;
use xiuxian_qianji_control::{
    ActivityId, ActivityType, HotStateStore, IdempotencyKey, InMemoryHotStateStore, RunId,
    RunnableActivityTask, StepId, TaskQueue, WorkerActivityTask,
};

#[tokio::test]
async fn claim_with_hot_state_returns_worker_task_and_lease_json() -> Result<(), String> {
    let hot_state = InMemoryHotStateStore::new();
    let run_id = must_ok(RunId::new("run-claim"), "should build run id");
    let step_id = must_ok(StepId::new("step-claim"), "should build step id");
    let task_queue = must_ok(TaskQueue::new("llm.openai"), "should build task queue");
    let activity_id = must_ok(
        ActivityId::new("activity-claim"),
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
                        IdempotencyKey::new("run-claim/activity-claim/1"),
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

    let output = must_ok(
        claim_with_hot_state(
            &hot_state,
            WorkerActivityClaimStoreRequest {
                worker_id: "worker-claim",
                task_queue: Some("llm.openai"),
                now_ms: 100,
                lease_ttl_ms: 50,
                json: true,
            },
        )
        .await,
        "activity claim should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "claim output should be valid json",
    );

    assert_eq!(json["worker_id"], "worker-claim");
    assert_eq!(json["task_queue"], "llm.openai");
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["run_id"],
        run_id.as_str()
    );
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["step_id"],
        step_id.as_str()
    );
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["activity_id"],
        activity_id.as_str()
    );
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["task_queue"],
        task_queue.as_str()
    );
    assert_eq!(json["claimed"]["lease"]["worker_id"], "worker-claim");
    assert_eq!(json["claimed"]["lease"]["expires_at_ms"], 150);
    Ok(())
}

#[tokio::test]
async fn claim_with_hot_state_renders_empty_text_when_no_task_matches() -> Result<(), String> {
    let hot_state = InMemoryHotStateStore::new();

    let output = must_ok(
        claim_with_hot_state(
            &hot_state,
            WorkerActivityClaimStoreRequest {
                worker_id: "worker-empty",
                task_queue: Some("tool.github"),
                now_ms: 10,
                lease_ttl_ms: 50,
                json: false,
            },
        )
        .await,
        "empty activity claim should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Control Activity Claim")
    );
    assert!(output.rendered.contains("- Worker: `worker-empty`"));
    assert!(output.rendered.contains("- Task queue: `tool.github`"));
    assert!(output.rendered.contains("- Claimed: `false`"));
    Ok(())
}

#[cfg(not(feature = "valkey"))]
#[test]
fn run_control_activity_claim_requires_valkey_feature_without_connecting() {
    let error = must_some(
        run_control_command(&ControlCliCommand::ActivityClaim {
            valkey_url: "redis://127.0.0.1:1".to_string(),
            namespace: None,
            worker_id: "worker-claim".to_string(),
            task_queue: Some("llm.openai".to_string()),
            now_ms: 10,
            lease_ttl_ms: 50,
            json: true,
        })
        .err(),
        "activity claim should require valkey feature in default tests",
    );

    assert!(
        error
            .to_string()
            .contains("`control activity-claim` requires the `valkey` feature"),
        "unexpected error: {error}"
    );
}
