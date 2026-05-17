use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, EvidenceGate, EvidenceId, EvidenceRef,
    HotStateStore, InMemoryControlLedger, InMemoryHotStateStore, RequiredEvidenceGate, RunId,
    RunnableStep, StepId, WorkerId, WorkerRef,
};

#[test]
fn in_memory_ledger_replays_required_evidence_gate() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-required-evidence")?;
    let step_id = StepId::new("validate-frontier")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "validate required evidence coverage".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Validate frontier".to_owned(),
            required_evidence: vec![
                "ownership_boundary".to_owned(),
                "validation_path".to_owned(),
            ],
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        3,
        ControlEventKind::EvidenceAttached {
            evidence: EvidenceRef {
                evidence_id: EvidenceId::new("evidence-validation-path")?,
                requirement_key: Some("validation_path".to_owned()),
                source: "unit-test".to_owned(),
                uri: None,
                summary: None,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;

    let view = ledger.load_run_view(&run_id)?;
    let step = view
        .steps
        .get(&step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;
    let gate = RequiredEvidenceGate::new("required-evidence")?;
    let result = gate.evaluate(step);

    assert!(!result.passed);
    assert_eq!(
        result.selected_required_evidence,
        vec!["validation_path".to_owned()]
    );
    assert_eq!(
        result.missing_required_evidence,
        vec!["ownership_boundary".to_owned()]
    );

    Ok(())
}

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
