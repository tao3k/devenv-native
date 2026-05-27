use std::error::Error;

use super::support::worker_ref;
use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, HotStateStore, InMemoryControlLedger,
    InMemoryHotStateStore, RecoveryActionApplication, RecoveryActionApplicationRequest,
    RecoveryItemScope, RecoveryPlanAction, RunId, StepId, TimerId, TimerRecord, TimerStatus,
    apply_recovery_action,
};

#[tokio::test]
async fn recovery_timer_applier_records_run_scoped_timer_fire() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-recovery-fire-run-timer")?;
    let timer_id = TimerId::new("approval-timeout")?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        100,
        ControlEventKind::TimerScheduled {
            timer: TimerRecord {
                timer_id: timer_id.clone(),
                fire_at_ms: 200,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    let action = RecoveryPlanAction::FireTimer {
        scope: RecoveryItemScope::run(),
        timer_id: timer_id.clone(),
        fire_at_ms: Some(200),
    };

    let result = apply_recovery_action(
        &ledger,
        &hot_state,
        RecoveryActionApplicationRequest::new(run_id.clone(), action, 250, 0),
    )
    .await?;

    assert!(matches!(
        result,
        RecoveryActionApplication::AppliedTimerFire { .. }
    ));
    let view = ledger.load_run_view(&run_id)?;
    let timer = view.timers.get(&timer_id).ok_or("missing fired timer")?;
    assert_eq!(timer.status, TimerStatus::Fired);
    assert_eq!(timer.fired_at_ms, Some(200));
    assert!(
        hot_state
            .acquire_lease(worker_ref()?, 250, 10)
            .await?
            .is_none(),
        "timer fire should not enqueue worker-visible steps"
    );
    Ok(())
}

#[tokio::test]
async fn recovery_timer_applier_records_step_scoped_timer_fire() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-recovery-fire-step-timer")?;
    let step_id = StepId::new("stage-wait")?;
    let timer_id = TimerId::new("stage-timeout")?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        100,
        ControlEventKind::TimerScheduled {
            timer: TimerRecord {
                timer_id: timer_id.clone(),
                fire_at_ms: 200,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    let action = RecoveryPlanAction::FireTimer {
        scope: RecoveryItemScope::step(step_id.clone()),
        timer_id: timer_id.clone(),
        fire_at_ms: None,
    };

    let result = apply_recovery_action(
        &ledger,
        &hot_state,
        RecoveryActionApplicationRequest::new(run_id.clone(), action, 250, 0),
    )
    .await?;

    assert!(matches!(
        result,
        RecoveryActionApplication::AppliedTimerFire { .. }
    ));
    let view = ledger.load_run_view(&run_id)?;
    let step = view.steps.get(&step_id).ok_or("missing step view")?;
    let timer = step.timers.get(&timer_id).ok_or("missing step timer")?;
    assert_eq!(timer.status, TimerStatus::Fired);
    assert_eq!(timer.fired_at_ms, Some(250));
    assert!(
        hot_state
            .acquire_lease(worker_ref()?, 250, 10)
            .await?
            .is_none(),
        "timer fire should not enqueue worker-visible steps"
    );
    Ok(())
}
