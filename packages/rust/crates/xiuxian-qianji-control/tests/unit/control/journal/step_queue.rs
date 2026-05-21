use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ControlLedger, HotStateStore, InMemoryControlLedger, InMemoryHotStateStore, RunId,
    RunnableStep, StepId, StepQueueJournalRecord, StepStatus, WorkerId, WorkerRef,
    record_step_queued, record_step_queued_with_hot_state,
};

#[test]
fn helper_records_step_queued_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let step = runnable_step("run-step-queue", "stage-plan", 100, 7)?;

    let record = record_step_queued(&ledger, StepQueueJournalRecord::new(step.clone(), 120))?;
    let view = ledger.load_run_view(&step.run_id)?;
    let step_view = view
        .steps
        .get(&step.step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;

    assert_eq!(record.sequence, 1);
    assert_eq!(step_view.status, StepStatus::Queued);
    Ok(())
}

#[tokio::test]
async fn helper_enqueues_hot_state_then_records_step_queued() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let step = runnable_step("run-step-queue-hot", "stage-plan-hot", 100, 7)?;

    let record = record_step_queued_with_hot_state(
        &ledger,
        &hot_state,
        StepQueueJournalRecord::new(step.clone(), 120),
    )
    .await?;
    let lease = hot_state
        .acquire_lease(
            WorkerRef {
                worker_id: WorkerId::new("worker-step-queue")?,
                capabilities: Vec::new(),
                metadata: serde_json::Value::Null,
            },
            100,
            10,
        )
        .await?
        .ok_or_else(|| io::Error::other("missing hot-state lease"))?;
    let view = ledger.load_run_view(&step.run_id)?;
    let step_view = view
        .steps
        .get(&step.step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;

    assert_eq!(record.sequence, 1);
    assert_eq!(lease.step_id, step.step_id);
    assert_eq!(step_view.status, StepStatus::Queued);
    Ok(())
}

fn runnable_step(
    run_id: &str,
    step_id: &str,
    not_before_ms: u64,
    priority: i64,
) -> Result<RunnableStep, Box<dyn Error>> {
    Ok(RunnableStep {
        run_id: RunId::new(run_id)?,
        step_id: StepId::new(step_id)?,
        priority,
        not_before_ms,
        metadata: serde_json::json!({"source": "unit"}),
    })
}
