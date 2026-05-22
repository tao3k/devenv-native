use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, ActivityType, HotStateStore, IdempotencyKey, InMemoryHotStateStore, RunId,
    RunnableActivityTask, RunnableStep, StepId, TaskQueue, WorkerActivityTask, WorkerHeartbeat,
    WorkerId, WorkerRef,
};

#[tokio::test]
async fn in_memory_hot_state_prefers_priority_and_requeues_expired_leases()
-> Result<(), Box<dyn Error>> {
    let store = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-hot-state")?;
    let low_priority_step_id = StepId::new("low-priority")?;
    let high_priority_step_id = StepId::new("high-priority")?;
    let worker = WorkerRef {
        worker_id: WorkerId::new("worker-a")?,
        capabilities: vec!["validation".to_owned()],
        metadata: serde_json::Value::Null,
    };

    store
        .enqueue_step(RunnableStep {
            run_id: run_id.clone(),
            step_id: low_priority_step_id,
            priority: 1,
            not_before_ms: 0,
            metadata: serde_json::Value::Null,
        })
        .await?;
    store
        .enqueue_step(RunnableStep {
            run_id,
            step_id: high_priority_step_id.clone(),
            priority: 10,
            not_before_ms: 0,
            metadata: serde_json::Value::Null,
        })
        .await?;

    let first_lease = store
        .acquire_lease(worker.clone(), 10, 10)
        .await?
        .ok_or_else(|| io::Error::other("missing first lease"))?;
    assert_eq!(first_lease.step_id, high_priority_step_id);

    let requeued_lease = store
        .acquire_lease(worker, 21, 10)
        .await?
        .ok_or_else(|| io::Error::other("missing requeued lease"))?;
    assert_eq!(requeued_lease.step_id, high_priority_step_id);

    Ok(())
}

#[tokio::test]
async fn in_memory_hot_state_snapshot_reports_queue_lease_and_heartbeat()
-> Result<(), Box<dyn Error>> {
    let store = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-hot-state-snapshot")?;
    let queued_step_id = StepId::new("queued")?;
    let leased_step_id = StepId::new("leased")?;
    let worker_id = WorkerId::new("worker-snapshot")?;
    let worker = WorkerRef {
        worker_id: worker_id.clone(),
        capabilities: vec!["audit".to_owned()],
        metadata: serde_json::Value::Null,
    };

    store
        .enqueue_step(RunnableStep {
            run_id: run_id.clone(),
            step_id: queued_step_id.clone(),
            priority: 1,
            not_before_ms: 50,
            metadata: serde_json::json!({"kind": "queued"}),
        })
        .await?;
    store
        .enqueue_step(RunnableStep {
            run_id,
            step_id: leased_step_id.clone(),
            priority: 10,
            not_before_ms: 0,
            metadata: serde_json::json!({"kind": "leased"}),
        })
        .await?;

    let lease = store
        .acquire_lease(worker, 100, 25)
        .await?
        .ok_or_else(|| io::Error::other("missing lease"))?;
    store
        .heartbeat(WorkerHeartbeat {
            worker_id: worker_id.clone(),
            observed_at_ms: 100,
            expires_at_ms: 150,
            metadata: serde_json::json!({"lane": "audit"}),
        })
        .await?;

    let snapshot = store.load_snapshot(130).await?;

    assert_eq!(snapshot.pending_steps.len(), 1);
    assert_eq!(snapshot.pending_steps[0].step_id, queued_step_id);
    assert_eq!(snapshot.leased_steps.len(), 1);
    assert_eq!(snapshot.leased_steps[0].step.step_id, leased_step_id);
    assert_eq!(snapshot.leased_steps[0].lease, lease);
    assert_eq!(snapshot.worker_heartbeats.len(), 1);
    assert_eq!(snapshot.worker_heartbeats[0].worker_id, worker_id);
    assert_eq!(snapshot.active_lease_count(), 0);
    assert_eq!(snapshot.expired_lease_count(), 1);
    assert_eq!(snapshot.live_heartbeat_count(), 1);
    Ok(())
}

#[tokio::test]
async fn in_memory_hot_state_claims_activity_tasks_by_queue_and_requeues_expired_leases()
-> Result<(), Box<dyn Error>> {
    let store = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-activity-hot-state")?;
    let llm_queue = TaskQueue::new("llm.openai")?;
    let tool_queue = TaskQueue::new("tool.github")?;
    let worker = WorkerRef {
        worker_id: WorkerId::new("worker-activity")?,
        capabilities: vec!["tool".to_owned()],
        metadata: serde_json::Value::Null,
    };

    store
        .enqueue_activity_task(RunnableActivityTask {
            task: worker_activity_task(&run_id, None, "activity-llm", &llm_queue)?,
            priority: 100,
            not_before_ms: 0,
            metadata: serde_json::Value::Null,
        })
        .await?;
    store
        .enqueue_activity_task(RunnableActivityTask {
            task: worker_activity_task(
                &run_id,
                Some(StepId::new("step-tool")?),
                "activity-tool",
                &tool_queue,
            )?,
            priority: 1,
            not_before_ms: 0,
            metadata: serde_json::Value::Null,
        })
        .await?;

    let first_lease = store
        .claim_activity_task(worker.clone(), Some(&tool_queue), 10, 10)
        .await?
        .ok_or_else(|| io::Error::other("missing first activity lease"))?;
    assert_eq!(first_lease.activity_task.task.task_queue, tool_queue);
    assert_eq!(
        first_lease.activity_task.task.activity_id,
        ActivityId::new("activity-tool")?
    );

    let no_tool_work = store
        .claim_activity_task(worker.clone(), Some(&tool_queue), 15, 10)
        .await?;
    assert!(no_tool_work.is_none());

    let requeued_lease = store
        .claim_activity_task(worker, Some(&tool_queue), 21, 10)
        .await?
        .ok_or_else(|| io::Error::other("missing requeued activity lease"))?;
    assert_eq!(requeued_lease.activity_task.task.task_queue, tool_queue);
    assert_ne!(requeued_lease.lease.lease_id, first_lease.lease.lease_id);
    Ok(())
}

#[tokio::test]
async fn in_memory_hot_state_snapshot_reports_activity_tasks() -> Result<(), Box<dyn Error>> {
    let store = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-activity-snapshot")?;
    let queue = TaskQueue::new("llm.openai")?;
    let worker = WorkerRef {
        worker_id: WorkerId::new("worker-activity-snapshot")?,
        capabilities: vec!["llm".to_owned()],
        metadata: serde_json::Value::Null,
    };

    store
        .enqueue_activity_task(RunnableActivityTask {
            task: worker_activity_task(&run_id, None, "activity-pending", &queue)?,
            priority: 1,
            not_before_ms: 200,
            metadata: serde_json::json!({"kind": "pending"}),
        })
        .await?;
    store
        .enqueue_activity_task(RunnableActivityTask {
            task: worker_activity_task(&run_id, None, "activity-leased", &queue)?,
            priority: 10,
            not_before_ms: 0,
            metadata: serde_json::json!({"kind": "leased"}),
        })
        .await?;

    let lease = store
        .claim_activity_task(worker, Some(&queue), 100, 25)
        .await?
        .ok_or_else(|| io::Error::other("missing activity lease"))?;
    let snapshot = store.load_snapshot(130).await?;

    assert_eq!(snapshot.pending_activity_tasks.len(), 1);
    assert_eq!(
        snapshot.pending_activity_tasks[0].task.activity_id,
        ActivityId::new("activity-pending")?
    );
    assert_eq!(snapshot.leased_activity_tasks.len(), 1);
    assert_eq!(
        snapshot.leased_activity_tasks[0]
            .activity_task
            .task
            .activity_id,
        ActivityId::new("activity-leased")?
    );
    assert_eq!(snapshot.leased_activity_tasks[0].lease, lease.lease);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    assert_eq!(snapshot.expired_activity_task_lease_count(), 1);
    Ok(())
}

#[tokio::test]
async fn in_memory_hot_state_releases_activity_task_leases() -> Result<(), Box<dyn Error>> {
    let store = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-activity-release")?;
    let queue = TaskQueue::new("llm.openai")?;
    let worker = WorkerRef {
        worker_id: WorkerId::new("worker-activity-release")?,
        capabilities: vec!["llm".to_owned()],
        metadata: serde_json::Value::Null,
    };

    store
        .enqueue_activity_task(RunnableActivityTask {
            task: worker_activity_task(&run_id, None, "activity-release", &queue)?,
            priority: 10,
            not_before_ms: 0,
            metadata: serde_json::Value::Null,
        })
        .await?;

    let leased = store
        .claim_activity_task(worker.clone(), Some(&queue), 100, 25)
        .await?
        .ok_or_else(|| io::Error::other("missing activity lease"))?;

    assert!(store.release_activity_task_lease(&leased.lease).await?);
    assert!(!store.release_activity_task_lease(&leased.lease).await?);

    let snapshot = store.load_snapshot(110).await?;
    assert!(snapshot.pending_activity_tasks.is_empty());
    assert!(snapshot.leased_activity_tasks.is_empty());
    assert!(
        store
            .claim_activity_task(worker, Some(&queue), 111, 25)
            .await?
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn in_memory_hot_state_reclaims_expired_activity_task_leases() -> Result<(), Box<dyn Error>> {
    let store = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-activity-reclaim")?;
    let queue = TaskQueue::new("tool.github")?;
    let worker = WorkerRef {
        worker_id: WorkerId::new("worker-activity-reclaim")?,
        capabilities: vec!["tool".to_owned()],
        metadata: serde_json::Value::Null,
    };

    store
        .enqueue_activity_task(RunnableActivityTask {
            task: worker_activity_task(
                &run_id,
                Some(StepId::new("step-a")?),
                "activity-a",
                &queue,
            )?,
            priority: 10,
            not_before_ms: 0,
            metadata: serde_json::Value::Null,
        })
        .await?;

    let leased = store
        .claim_activity_task(worker.clone(), Some(&queue), 100, 25)
        .await?
        .ok_or_else(|| io::Error::other("missing activity lease"))?;

    assert!(
        !store
            .reclaim_expired_activity_task_lease(&leased.lease, 124)
            .await?
    );
    assert!(
        store
            .reclaim_expired_activity_task_lease(&leased.lease, 125)
            .await?
    );
    assert!(
        !store
            .reclaim_expired_activity_task_lease(&leased.lease, 126)
            .await?
    );

    let requeued = store
        .claim_activity_task(worker, Some(&queue), 126, 25)
        .await?
        .ok_or_else(|| io::Error::other("missing requeued activity lease"))?;
    assert_eq!(
        requeued.activity_task.task.activity_id,
        ActivityId::new("activity-a")?
    );
    assert_ne!(requeued.lease.lease_id, leased.lease.lease_id);
    Ok(())
}

fn worker_activity_task(
    run_id: &RunId,
    step_id: Option<StepId>,
    activity_id: &str,
    task_queue: &TaskQueue,
) -> Result<WorkerActivityTask, Box<dyn Error>> {
    Ok(WorkerActivityTask {
        run_id: run_id.clone(),
        step_id,
        activity_id: ActivityId::new(activity_id)?,
        activity_type: ActivityType::new("llm.plan")?,
        task_queue: task_queue.clone(),
        next_attempt: 1,
        scheduled_at_ms: 10,
        input_ref: None,
        idempotency_key: IdempotencyKey::new(format!("{activity_id}/attempt/1"))?,
        retry_policy: None,
        timeout_ms: Some(30_000),
        metadata: serde_json::Value::Null,
    })
}
