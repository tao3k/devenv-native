use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    HotStateStore, InMemoryHotStateStore, RunId, RunnableStep, StepId, WorkerHeartbeat, WorkerId,
    WorkerRef,
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
