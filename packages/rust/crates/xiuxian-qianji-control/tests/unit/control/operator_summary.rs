use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, ControlEvent, ControlEventKind, ControlLedger,
    CostObservation, IdempotencyKey, InMemoryControlLedger, LeaseId, RunId, SignalName, StepId,
    StepLease, TaskQueue, TimerId, TimerRecord, WorkerId,
};

#[test]
fn operator_summary_aggregates_durable_management_counters() -> Result<(), Box<dyn Error>> {
    let ledger = operator_summary_fixture()?;
    let run_id = RunId::new("run-operator-summary")?;
    let summary = ledger.load_operator_summary(&run_id, 15_000)?;

    assert_eq!(summary.run_id, run_id);
    assert_eq!(summary.observed_at_ms, 15_000);
    assert_eq!(summary.event_count, 7);
    assert_eq!(summary.steps, 1);
    assert_eq!(summary.active_leases, 1);
    assert_eq!(summary.activities.total, 1);
    assert_eq!(summary.activities.scheduled, 1);
    assert_eq!(summary.timers.total, 1);
    assert_eq!(summary.timers.scheduled, 1);
    assert_eq!(summary.signals.total, 1);
    assert_eq!(summary.signals.step_scoped, 1);
    assert_eq!(summary.costs.total_tokens, 42);
    assert_eq!(summary.costs.cost_usd_micros, 130);
    assert_eq!(summary.recovery.reclaim_expired_leases, 1);
    assert_eq!(summary.recovery.fireable_timers, 1);
    Ok(())
}

fn operator_summary_fixture() -> Result<InMemoryControlLedger, Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-operator-summary")?;
    let step_id = StepId::new("step-operator-summary")?;
    let activity_id = ActivityId::new("activity-operator-summary")?;
    let timer_id = TimerId::new("timer-operator-summary")?;
    let lease = StepLease {
        lease_id: LeaseId::new("lease-operator-summary")?,
        run_id: run_id.clone(),
        step_id: step_id.clone(),
        worker_id: WorkerId::new("worker-operator-summary")?,
        acquired_at_ms: 10_000,
        expires_at_ms: 14_000,
    };

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "operator summary projection".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Inspect durable state".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        10_000,
        ControlEventKind::StepLeaseAcquired { lease },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        11_000,
        ControlEventKind::ActivityScheduled {
            task: activity_task(activity_id)?,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        12_000,
        ControlEventKind::TimerScheduled {
            timer: TimerRecord {
                timer_id,
                fire_at_ms: 13_000,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id,
        12_500,
        ControlEventKind::SignalReceived {
            signal: xiuxian_qianji_control::SignalRecord {
                signal_name: SignalName::new("human.approval")?,
                payload_ref: None,
                payload_hash: None,
                metadata: serde_json::json!({"approved": true}),
            },
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id,
        12_750,
        ControlEventKind::CostObserved {
            observation: CostObservation {
                provider: "llm.openai".to_owned(),
                model: Some("gpt-test".to_owned()),
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: Some(42),
                cost_usd_micros: 130,
                latency_ms: Some(1_250),
            },
        },
    ))?;
    Ok(ledger)
}

fn activity_task(activity_id: ActivityId) -> Result<ActivityTask, Box<dyn Error>> {
    Ok(ActivityTask::new(
        activity_id,
        ActivityType::new("llm.plan")?,
        TaskQueue::new("llm.openai")?,
        IdempotencyKey::new("operator-summary-activity")?,
    ))
}
