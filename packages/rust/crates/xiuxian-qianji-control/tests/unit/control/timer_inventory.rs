use std::error::Error;

use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, InMemoryControlLedger, RecoveryItemScope, RunId,
    StepId, TimerId, TimerRecord,
};

#[test]
fn timer_inventory_projection_surfaces_run_and_step_timers() -> Result<(), Box<dyn Error>> {
    let ledger = timer_inventory_fixture()?;
    let run_id = RunId::new("run-timer-inventory")?;
    let projection = ledger.load_timer_inventory_projection(&run_id)?;

    assert_eq!(projection.run_id, run_id);
    assert_eq!(projection.items.len(), 2);
    assert_eq!(projection.summary.total, 2);
    assert_eq!(projection.summary.pending, 0);
    assert_eq!(projection.summary.scheduled, 1);
    assert_eq!(projection.summary.fired, 1);
    assert_eq!(
        projection.items[0].timer.timer_id,
        TimerId::new("timer-run-wakeup")?
    );
    assert_eq!(projection.items[0].scope, RecoveryItemScope::run());
    assert_eq!(
        projection.items[1].timer.timer_id,
        TimerId::new("timer-step-approval-timeout")?
    );
    assert_eq!(
        projection.items[1].scope,
        RecoveryItemScope::step(StepId::new("step-timer-inventory")?)
    );
    Ok(())
}

fn timer_inventory_fixture() -> Result<InMemoryControlLedger, Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-timer-inventory")?;
    let step_id = StepId::new("step-timer-inventory")?;
    let run_timer_id = TimerId::new("timer-run-wakeup")?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "timer inventory projection".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Wait for approval".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        3,
        ControlEventKind::TimerScheduled {
            timer: TimerRecord {
                timer_id: run_timer_id.clone(),
                fire_at_ms: 10_000,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        10_250,
        ControlEventKind::TimerFired {
            timer_id: run_timer_id,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id,
        step_id,
        4,
        ControlEventKind::TimerScheduled {
            timer: TimerRecord {
                timer_id: TimerId::new("timer-step-approval-timeout")?,
                fire_at_ms: 20_000,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    Ok(ledger)
}
