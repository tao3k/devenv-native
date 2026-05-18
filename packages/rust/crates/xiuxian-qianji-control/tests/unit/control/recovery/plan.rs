use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityRetryDecision, ActivityRetryPolicy, ActivityRetryStopReason, AgentDecision,
    AgentDecisionId, AgentDecisionOutcome, AgentProposalId, ControlEvent, ControlEventKind,
    ControlLedger, DecisionReasonCode, InMemoryControlLedger, RecoveryItemScope,
    RecoveryPlanAction, RunId, RunRecoveryPlanSummary, StepId, TimerId, WaitReason, WorkerId,
};

use super::support::{
    activity_failure, append_run_created, append_step_created, step_lease, timer_record,
};
use crate::control::support::activity_task;

struct PlanFixture {
    run_id: RunId,
    ids: PlanFixtureIds,
    actions: Vec<RecoveryPlanAction>,
}

struct PlanFixtureIds {
    active_step: StepId,
    expired_step: StepId,
    human_step: StepId,
    blocked_step: StepId,
    scheduled: ActivityId,
    in_flight: ActivityId,
    retry: ActivityId,
    review: ActivityId,
    terminal: ActivityId,
    fireable_timer: TimerId,
    pending_timer: TimerId,
    approval_decision: AgentDecisionId,
}

#[test]
fn recovery_plan_projects_ordered_actions_from_recovery_view() -> Result<(), Box<dyn Error>> {
    let fixture = build_plan_fixture()?;

    assert_eq!(fixture.run_id.as_str(), "run-recovery-plan");
    assert!(!fixture.actions.is_empty());
    assert_eq!(fixture.actions, expected_plan_actions(&fixture.ids)?);

    Ok(())
}

#[test]
fn control_ledger_load_recovery_plan_matches_manual_projection() -> Result<(), Box<dyn Error>> {
    let fixture = build_plan_ledger_fixture()?;
    let manual = fixture
        .ledger
        .load_run_view(&fixture.run_id)?
        .recovery_view(10_000)?
        .recovery_plan();
    let event_count_before = fixture.ledger.load_events(&fixture.run_id)?.len();

    let helper = fixture.ledger.load_recovery_plan(&fixture.run_id, 10_000)?;

    assert_eq!(helper, manual);
    assert_eq!(helper.actions, expected_plan_actions(&fixture.ids)?);
    assert_eq!(
        fixture.ledger.load_events(&fixture.run_id)?.len(),
        event_count_before
    );

    Ok(())
}

#[test]
fn recovery_plan_summary_counts_action_kinds() -> Result<(), Box<dyn Error>> {
    let fixture = build_plan_ledger_fixture()?;

    let summary = fixture
        .ledger
        .load_recovery_plan(&fixture.run_id, 10_000)?
        .summary();

    assert_eq!(summary, expected_plan_summary());

    Ok(())
}

#[test]
fn recovery_plan_snapshot_packages_view_plan_and_summary() -> Result<(), Box<dyn Error>> {
    let fixture = build_plan_ledger_fixture()?;
    let event_count_before = fixture.ledger.load_events(&fixture.run_id)?.len();

    let snapshot = fixture
        .ledger
        .load_recovery_snapshot(&fixture.run_id, 10_000)?;

    assert_eq!(snapshot.run_id, fixture.run_id);
    assert_eq!(snapshot.observed_at_ms, 10_000);
    assert!(snapshot.view.has_recovery_work());
    assert_eq!(snapshot.plan.actions, expected_plan_actions(&fixture.ids)?);
    assert_eq!(snapshot.summary, expected_plan_summary());
    assert_eq!(snapshot.summary, snapshot.plan.summary());
    assert_eq!(
        fixture.ledger.load_events(&snapshot.run_id)?.len(),
        event_count_before
    );

    Ok(())
}

#[test]
fn recovery_plan_is_empty_when_recovery_view_has_no_work() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-recovery-plan-empty")?;

    append_run_created(&ledger, run_id.clone())?;

    let plan = ledger
        .load_run_view(&run_id)?
        .recovery_view(10_000)?
        .recovery_plan();

    assert_eq!(plan.run_id, run_id);
    assert_eq!(plan.planned_at_ms, 10_000);
    assert!(!plan.has_actions());
    assert!(plan.actions.is_empty());
    assert_eq!(plan.summary(), RunRecoveryPlanSummary::default());

    Ok(())
}

struct PlanLedgerFixture {
    ledger: InMemoryControlLedger,
    run_id: RunId,
    ids: PlanFixtureIds,
}

fn build_plan_fixture() -> Result<PlanFixture, Box<dyn Error>> {
    let fixture = build_plan_ledger_fixture()?;
    let plan = fixture.ledger.load_recovery_plan(&fixture.run_id, 10_000)?;

    assert!(plan.has_actions());
    assert_eq!(plan.run_id, fixture.run_id);
    assert_eq!(plan.planned_at_ms, 10_000);
    Ok(PlanFixture {
        run_id: fixture.run_id,
        ids: fixture.ids,
        actions: plan.actions,
    })
}

fn build_plan_ledger_fixture() -> Result<PlanLedgerFixture, Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-recovery-plan")?;
    let ids = PlanFixtureIds {
        active_step: StepId::new("step-active-lease")?,
        expired_step: StepId::new("step-expired-lease")?,
        human_step: StepId::new("step-human-wait")?,
        blocked_step: StepId::new("step-blocked")?,
        scheduled: ActivityId::new("activity-scheduled")?,
        in_flight: ActivityId::new("activity-in-flight")?,
        retry: ActivityId::new("activity-retry-policy")?,
        review: ActivityId::new("activity-retry-review")?,
        terminal: ActivityId::new("activity-terminal")?,
        fireable_timer: TimerId::new("timer-fireable")?,
        pending_timer: TimerId::new("timer-pending")?,
        approval_decision: AgentDecisionId::new("decision-approval")?,
    };

    append_run_created(&ledger, run_id.clone())?;
    append_step_created(&ledger, run_id.clone(), ids.active_step.clone(), 2)?;
    append_step_created(&ledger, run_id.clone(), ids.expired_step.clone(), 3)?;
    append_step_created(&ledger, run_id.clone(), ids.human_step.clone(), 4)?;
    append_step_created(&ledger, run_id.clone(), ids.blocked_step.clone(), 5)?;
    append_leases(&ledger, &run_id, &ids.active_step, &ids.expired_step)?;
    append_timers(
        &ledger,
        &run_id,
        ids.fireable_timer.clone(),
        ids.pending_timer.clone(),
    )?;
    append_activities(
        &ledger,
        &run_id,
        ids.scheduled.clone(),
        ids.in_flight.clone(),
        ids.retry.clone(),
        ids.review.clone(),
        ids.terminal.clone(),
    )?;
    append_human_and_blocked_steps(
        &ledger,
        &run_id,
        &ids.human_step,
        &ids.blocked_step,
        ids.approval_decision.clone(),
    )?;

    Ok(PlanLedgerFixture {
        ledger,
        run_id,
        ids,
    })
}

fn expected_plan_actions(ids: &PlanFixtureIds) -> Result<Vec<RecoveryPlanAction>, Box<dyn Error>> {
    Ok(vec![
        RecoveryPlanAction::ReclaimExpiredLease {
            step_id: ids.expired_step.clone(),
            lease_id: "lease-expired".try_into()?,
        },
        RecoveryPlanAction::FireTimer {
            scope: RecoveryItemScope::run(),
            timer_id: ids.fireable_timer.clone(),
            fire_at_ms: Some(9_000),
        },
        RecoveryPlanAction::RetryActivity {
            scope: RecoveryItemScope::run(),
            activity_id: ids.retry.clone(),
            retry_decision: ActivityRetryDecision::Retry {
                next_attempt: 2,
                backoff_ms: 100,
            },
        },
        RecoveryPlanAction::ReviewRetryableActivity {
            scope: RecoveryItemScope::run(),
            activity_id: ids.review.clone(),
        },
        RecoveryPlanAction::EscalateTerminalActivity {
            scope: RecoveryItemScope::run(),
            activity_id: ids.terminal.clone(),
            retry_decision: Some(ActivityRetryDecision::DoNotRetry {
                reason: ActivityRetryStopReason::AttemptsExhausted,
            }),
        },
        RecoveryPlanAction::ReconcileScheduledActivity {
            scope: RecoveryItemScope::run(),
            activity_id: ids.scheduled.clone(),
        },
        RecoveryPlanAction::InspectInFlightActivity {
            scope: RecoveryItemScope::run(),
            activity_id: ids.in_flight.clone(),
        },
        RecoveryPlanAction::AwaitHumanApproval {
            scope: RecoveryItemScope::step(ids.human_step.clone()),
            decision_id: ids.approval_decision.clone(),
        },
        RecoveryPlanAction::AwaitHumanInput {
            step_id: ids.human_step.clone(),
        },
        RecoveryPlanAction::InspectBlockedStep {
            step_id: ids.blocked_step.clone(),
            last_error: Some("required evidence is missing".to_owned()),
        },
        RecoveryPlanAction::PreserveActiveLease {
            step_id: ids.active_step.clone(),
            lease_id: "lease-active".try_into()?,
        },
        RecoveryPlanAction::AwaitTimer {
            scope: RecoveryItemScope::run(),
            timer_id: ids.pending_timer.clone(),
            fire_at_ms: Some(20_000),
        },
    ])
}

fn expected_plan_summary() -> RunRecoveryPlanSummary {
    RunRecoveryPlanSummary {
        total_actions: 12,
        reclaim_expired_leases: 1,
        fireable_timers: 1,
        retry_activities: 1,
        review_retryable_activities: 1,
        terminal_activity_escalations: 1,
        reconcile_scheduled_activities: 1,
        inspect_in_flight_activities: 1,
        await_human_approvals: 1,
        await_human_inputs: 1,
        inspect_blocked_steps: 1,
        preserve_active_leases: 1,
        await_timers: 1,
    }
}

fn append_leases(
    ledger: &InMemoryControlLedger,
    run_id: &RunId,
    active_step: &StepId,
    expired_step: &StepId,
) -> Result<(), Box<dyn Error>> {
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        active_step.clone(),
        6,
        ControlEventKind::StepLeaseAcquired {
            lease: step_lease(run_id, active_step, "lease-active", 1_000, 20_000)?,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        expired_step.clone(),
        7,
        ControlEventKind::StepLeaseAcquired {
            lease: step_lease(run_id, expired_step, "lease-expired", 1_000, 5_000)?,
        },
    ))?;
    Ok(())
}

fn append_timers(
    ledger: &InMemoryControlLedger,
    run_id: &RunId,
    fireable_timer: TimerId,
    pending_timer: TimerId,
) -> Result<(), Box<dyn Error>> {
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        8,
        ControlEventKind::TimerScheduled {
            timer: timer_record(fireable_timer, 9_000),
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        9,
        ControlEventKind::TimerScheduled {
            timer: timer_record(pending_timer, 20_000),
        },
    ))?;
    Ok(())
}

fn append_activities(
    ledger: &InMemoryControlLedger,
    run_id: &RunId,
    scheduled: ActivityId,
    in_flight: ActivityId,
    retry: ActivityId,
    review: ActivityId,
    terminal: ActivityId,
) -> Result<(), Box<dyn Error>> {
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        10,
        ControlEventKind::ActivityScheduled {
            task: activity_task(scheduled)?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        11,
        ControlEventKind::ActivityScheduled {
            task: activity_task(in_flight.clone())?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        12,
        ControlEventKind::ActivityStarted {
            activity_id: in_flight,
            worker_id: Some(WorkerId::new("worker-in-flight")?),
            attempt: 1,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        13,
        ControlEventKind::ActivityScheduled {
            task: activity_task(retry.clone())?.with_retry_policy(
                ActivityRetryPolicy::new(3)?
                    .with_initial_interval_ms(100)
                    .with_max_interval_ms(1_000),
            ),
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        14,
        ControlEventKind::ActivityFailed {
            activity_id: retry,
            failure: activity_failure("rate-limit", true, 1)?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        15,
        ControlEventKind::ActivityScheduled {
            task: activity_task(review.clone())?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        16,
        ControlEventKind::ActivityFailed {
            activity_id: review,
            failure: activity_failure("retryable-no-policy", true, 1)?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        17,
        ControlEventKind::ActivityScheduled {
            task: activity_task(terminal.clone())?
                .with_retry_policy(ActivityRetryPolicy::new(1)?.with_initial_interval_ms(100)),
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        18,
        ControlEventKind::ActivityFailed {
            activity_id: terminal,
            failure: activity_failure("timeout", true, 1)?,
        },
    ))?;
    Ok(())
}

fn append_human_and_blocked_steps(
    ledger: &InMemoryControlLedger,
    run_id: &RunId,
    human_step: &StepId,
    blocked_step: &StepId,
    approval_decision: AgentDecisionId,
) -> Result<(), Box<dyn Error>> {
    let decision = AgentDecision::new(
        approval_decision,
        AgentProposalId::new("proposal-approval")?,
        AgentDecisionOutcome::ApprovalRequired,
        DecisionReasonCode::new("human_approval_required")?,
    );
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        human_step.clone(),
        19,
        ControlEventKind::StepWaiting {
            reason: WaitReason::Human,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        blocked_step.clone(),
        20,
        ControlEventKind::StepBlocked {
            reason: "required evidence is missing".to_owned(),
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        human_step.clone(),
        21,
        ControlEventKind::AgentDecisionRecorded { decision },
    ))?;
    Ok(())
}
