use std::error::Error;

use xiuxian_qianji_control::{
    ControlError, ControlLedger, HotStateStore, InMemoryControlLedger, InMemoryHotStateStore,
    RunId, WorkerHeartbeat, WorkerHeartbeatJournalRecord, WorkerId, record_worker_heartbeat,
    record_worker_heartbeat_with_hot_state,
};

#[test]
fn helper_records_worker_heartbeat_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-worker-heartbeat")?;
    let heartbeat = worker_heartbeat("worker-heartbeat", 100, 250)?;

    let record = record_worker_heartbeat(
        &ledger,
        WorkerHeartbeatJournalRecord::new(run_id.clone(), heartbeat),
    )?;
    let records = ledger.load_events(&run_id)?;
    let view = ledger.load_run_view(&run_id)?;

    assert_eq!(record.sequence, 1);
    assert_eq!(records.len(), 1);
    assert_eq!(view.worker_heartbeats.len(), 1);
    assert_eq!(
        view.worker_heartbeats[0].worker_id.as_str(),
        "worker-heartbeat"
    );
    assert_eq!(records[0].event.occurred_at_ms, 100);
    assert_eq!(
        serde_json::to_value(&records[0].event.kind)?["heartbeat"]["worker_id"],
        "worker-heartbeat"
    );
    Ok(())
}

#[tokio::test]
async fn helper_records_worker_heartbeat_in_hot_state_then_ledger() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-worker-heartbeat-hot")?;
    let heartbeat = worker_heartbeat("worker-heartbeat-hot", 100, 250)?;

    let record = record_worker_heartbeat_with_hot_state(
        &ledger,
        &hot_state,
        WorkerHeartbeatJournalRecord::new(run_id.clone(), heartbeat.clone()),
    )
    .await?;
    let loaded = hot_state
        .load_heartbeat(&heartbeat.worker_id)
        .await?
        .ok_or("missing hot heartbeat")?;
    let records = ledger.load_events(&run_id)?;

    assert_eq!(record.sequence, 1);
    assert_eq!(loaded, heartbeat);
    assert_eq!(records.len(), 1);
    Ok(())
}

#[test]
fn helper_rejects_expired_worker_heartbeat_without_append() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-worker-heartbeat-expired")?;
    let heartbeat = worker_heartbeat("worker-heartbeat-expired", 100, 100)?;

    let Err(error) = record_worker_heartbeat(
        &ledger,
        WorkerHeartbeatJournalRecord::new(run_id.clone(), heartbeat),
    ) else {
        panic!("expired heartbeat should fail");
    };
    let records = ledger.load_events(&run_id)?;

    assert!(matches!(
        error,
        ControlError::Storage {
            operation: "validate_worker_heartbeat_ttl",
            ..
        }
    ));
    assert!(records.is_empty());
    Ok(())
}

fn worker_heartbeat(
    worker_id: &str,
    observed_at_ms: u64,
    expires_at_ms: u64,
) -> Result<WorkerHeartbeat, Box<dyn Error>> {
    Ok(WorkerHeartbeat {
        worker_id: WorkerId::new(worker_id)?,
        observed_at_ms,
        expires_at_ms,
        metadata: serde_json::json!({"queue": "llm.openai"}),
    })
}
