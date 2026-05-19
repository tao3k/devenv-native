use std::error::Error;

use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, InMemoryControlLedger, InMemoryHotStateStore,
    RecoveryActionApplication, RecoveryAttempt, RecoveryItemScope, RecoveryLoopApplicationRequest,
    RecoveryPlanAction, RecoveryPolicy, RunId, RunRecoveryPlan, RunStatus, TimerId, TimerRecord,
    TimerStatus, apply_recovery_plan,
};

#[tokio::test]
async fn recovery_loop_records_attempt_and_applies_actions_in_order() -> Result<(), Box<dyn Error>>
{
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-bounded-recovery-loop")?;
    let timer_id = TimerId::new("ready-timer")?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        10,
        ControlEventKind::TimerScheduled {
            timer: TimerRecord {
                timer_id: timer_id.clone(),
                fire_at_ms: 100,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    let plan = RunRecoveryPlan {
        run_id: run_id.clone(),
        planned_at_ms: 120,
        actions: vec![
            RecoveryPlanAction::FireTimer {
                scope: RecoveryItemScope::run(),
                timer_id: timer_id.clone(),
                fire_at_ms: Some(100),
            },
            RecoveryPlanAction::AwaitTimer {
                scope: RecoveryItemScope::run(),
                timer_id: TimerId::new("not-ready-timer")?,
                fire_at_ms: Some(500),
            },
        ],
    };

    let result = apply_recovery_plan(
        &ledger,
        &hot_state,
        RecoveryLoopApplicationRequest::new(plan, recovery_attempt(), 120, 0),
    )
    .await?;
    let events = ledger.load_events(&run_id)?;
    let view = ledger.load_run_view(&run_id)?;
    let timer = view.timers.get(&timer_id).ok_or("missing fired timer")?;

    assert_eq!(result.attempt_record.sequence, 2);
    assert_eq!(result.action_results.len(), 2);
    assert!(matches!(
        result.action_results[0].result,
        RecoveryActionApplication::AppliedTimerFire { .. }
    ));
    assert!(matches!(
        result.action_results[1].result,
        RecoveryActionApplication::NotApplicable { .. }
    ));
    assert_eq!(events.len(), 3);
    assert!(matches!(
        events[1].event.kind,
        ControlEventKind::RecoveryStarted { .. }
    ));
    assert!(matches!(
        events[2].event.kind,
        ControlEventKind::TimerFired { .. }
    ));
    assert_eq!(view.status, RunStatus::Recovering);
    assert_eq!(timer.status, TimerStatus::Fired);
    assert_eq!(timer.fired_at_ms, Some(120));
    Ok(())
}

fn recovery_attempt() -> RecoveryAttempt {
    RecoveryAttempt {
        attempt: 1,
        reason: "bounded recovery loop".to_owned(),
        policy: RecoveryPolicy {
            max_attempts: 3,
            backoff_ms: 100,
            require_human_approval: false,
        },
    }
}
