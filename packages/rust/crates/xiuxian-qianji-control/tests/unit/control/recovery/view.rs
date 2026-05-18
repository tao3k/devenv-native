use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, ActivityRetryDecision, ActivityRetryPolicy, ActivityRetryStopReason, AgentDecision,
    AgentDecisionId, AgentDecisionOutcome, AgentProposalId, ControlEvent, ControlEventKind,
    ControlLedger, DecisionReasonCode, InMemoryControlLedger, RecoveryItemScope, RunId, StepId,
    TimerId, WaitReason, WorkerId,
};

use crate::control::recovery::support::{
    activity_failure, append_run_created, append_step_created, step_lease, timer_record,
};
use crate::control::support::activity_task;

#[test]
fn recovery_view_classifies_activity_recovery_work() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-recovery-activities")?;
    let scheduled_id = ActivityId::new("activity-scheduled")?;
    let in_flight_id = ActivityId::new("activity-in-flight")?;
    let retryable_id = ActivityId::new("activity-retryable")?;
    let terminal_id = ActivityId::new("activity-terminal")?;

    append_run_created(&ledger, run_id.clone())?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        2,
        ControlEventKind::ActivityScheduled {
            task: activity_task(scheduled_id.clone())?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        3,
        ControlEventKind::ActivityScheduled {
            task: activity_task(in_flight_id.clone())?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        4,
        ControlEventKind::ActivityStarted {
            activity_id: in_flight_id.clone(),
            worker_id: Some(WorkerId::new("worker-in-flight")?),
            attempt: 1,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        5,
        ControlEventKind::ActivityScheduled {
            task: activity_task(retryable_id.clone())?.with_retry_policy(
                ActivityRetryPolicy::new(3)?
                    .with_initial_interval_ms(100)
                    .with_max_interval_ms(1_000),
            ),
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        6,
        ControlEventKind::ActivityFailed {
            activity_id: retryable_id.clone(),
            failure: activity_failure("rate-limit", true, 1)?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        7,
        ControlEventKind::ActivityScheduled {
            task: activity_task(terminal_id.clone())?
                .with_retry_policy(ActivityRetryPolicy::new(1)?.with_initial_interval_ms(100)),
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        8,
        ControlEventKind::ActivityFailed {
            activity_id: terminal_id.clone(),
            failure: activity_failure("timeout", true, 1)?,
        },
    ))?;

    let recovery = ledger.load_run_view(&run_id)?.recovery_view(10_000)?;

    assert!(recovery.has_recovery_work());
    assert_eq!(
        recovery.scheduled_activities[0].activity.activity_id,
        scheduled_id
    );
    assert_eq!(
        recovery.in_flight_activities[0].activity.activity_id,
        in_flight_id
    );
    assert_eq!(
        recovery.retryable_failed_activities[0].activity.activity_id,
        retryable_id
    );
    assert_eq!(
        recovery.retryable_failed_activities[0].retry_decision,
        Some(ActivityRetryDecision::Retry {
            next_attempt: 2,
            backoff_ms: 100,
        })
    );
    assert_eq!(
        recovery.terminal_failed_activities[0].activity.activity_id,
        terminal_id
    );
    assert_eq!(
        recovery.terminal_failed_activities[0].retry_decision,
        Some(ActivityRetryDecision::DoNotRetry {
            reason: ActivityRetryStopReason::AttemptsExhausted,
        })
    );

    Ok(())
}

#[test]
fn recovery_view_classifies_timers_and_leases() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-recovery-timers-leases")?;
    let active_step_id = StepId::new("step-active-lease")?;
    let expired_step_id = StepId::new("step-expired-lease")?;
    let pending_timer_id = TimerId::new("timer-pending")?;
    let fireable_timer_id = TimerId::new("timer-fireable")?;

    append_run_created(&ledger, run_id.clone())?;
    append_step_created(&ledger, run_id.clone(), active_step_id.clone(), 2)?;
    append_step_created(&ledger, run_id.clone(), expired_step_id.clone(), 3)?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        active_step_id.clone(),
        4,
        ControlEventKind::StepLeaseAcquired {
            lease: step_lease(&run_id, &active_step_id, "lease-active", 1_000, 20_000)?,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        expired_step_id.clone(),
        5,
        ControlEventKind::StepLeaseAcquired {
            lease: step_lease(&run_id, &expired_step_id, "lease-expired", 1_000, 5_000)?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        6,
        ControlEventKind::TimerScheduled {
            timer: timer_record(pending_timer_id.clone(), 20_000),
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        7,
        ControlEventKind::TimerScheduled {
            timer: timer_record(fireable_timer_id.clone(), 9_000),
        },
    ))?;

    let recovery = ledger.load_run_view(&run_id)?.recovery_view(10_000)?;

    assert!(recovery.has_recovery_work());
    assert_eq!(recovery.active_leases[0].step_id, active_step_id);
    assert_eq!(recovery.expired_leases[0].step_id, expired_step_id);
    assert_eq!(recovery.pending_timers[0].timer.timer_id, pending_timer_id);
    assert_eq!(
        recovery.fireable_timers[0].timer.timer_id,
        fireable_timer_id
    );
    assert_eq!(recovery.pending_timers[0].scope, RecoveryItemScope::run());
    assert_eq!(recovery.fireable_timers[0].scope, RecoveryItemScope::run());

    Ok(())
}

#[test]
fn recovery_view_surfaces_approval_waits_and_blocked_steps() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-recovery-approval")?;
    let human_step_id = StepId::new("step-human-wait")?;
    let blocked_step_id = StepId::new("step-blocked")?;
    let decision = AgentDecision::new(
        AgentDecisionId::new("decision-approval")?,
        AgentProposalId::new("proposal-approval")?,
        AgentDecisionOutcome::ApprovalRequired,
        DecisionReasonCode::new("human_approval_required")?,
    );

    append_run_created(&ledger, run_id.clone())?;
    append_step_created(&ledger, run_id.clone(), human_step_id.clone(), 2)?;
    append_step_created(&ledger, run_id.clone(), blocked_step_id.clone(), 3)?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        human_step_id.clone(),
        4,
        ControlEventKind::StepWaiting {
            reason: WaitReason::Human,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        blocked_step_id.clone(),
        5,
        ControlEventKind::StepBlocked {
            reason: "required evidence is missing".to_owned(),
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        human_step_id.clone(),
        6,
        ControlEventKind::AgentDecisionRecorded { decision },
    ))?;

    let recovery = ledger.load_run_view(&run_id)?.recovery_view(10_000)?;

    assert_eq!(recovery.pending_approval_decisions.len(), 1);
    assert_eq!(
        recovery.pending_approval_decisions[0].scope,
        RecoveryItemScope::step(human_step_id.clone())
    );
    assert_eq!(recovery.human_wait_steps[0].step_id, human_step_id);
    assert_eq!(
        recovery.human_wait_steps[0].wait_reason,
        Some(WaitReason::Human)
    );
    assert_eq!(recovery.blocked_steps[0].step_id, blocked_step_id);
    assert_eq!(
        recovery.blocked_steps[0].last_error.as_deref(),
        Some("required evidence is missing")
    );

    Ok(())
}

#[test]
fn recovery_view_rejects_invalid_replayed_retry_failure() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-recovery-invalid-retry")?;
    let activity_id = ActivityId::new("activity-invalid-retry")?;

    append_run_created(&ledger, run_id.clone())?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        2,
        ControlEventKind::ActivityScheduled {
            task: activity_task(activity_id.clone())?
                .with_retry_policy(ActivityRetryPolicy::new(2)?.with_initial_interval_ms(100)),
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        3,
        ControlEventKind::ActivityFailed {
            activity_id,
            failure: activity_failure("invalid-attempt", true, 0)?,
        },
    ))?;

    let Err(error) = ledger.load_run_view(&run_id)?.recovery_view(10_000) else {
        return Err(
            io::Error::other("invalid retry failure should fail recovery derivation").into(),
        );
    };

    assert!(
        error.to_string().contains("attempt"),
        "unexpected error: {error}"
    );

    Ok(())
}
