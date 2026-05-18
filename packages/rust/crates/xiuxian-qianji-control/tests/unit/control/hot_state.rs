use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    HotStateStore, InMemoryHotStateStore, RunId, RunnableStep, StepId, WorkerId, WorkerRef,
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
