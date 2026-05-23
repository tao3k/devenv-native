use std::error::Error;

use xiuxian_qianji_control::{
    ActivityFailure, ActivityId, ActivityRetryDecision, ActivityRetryPolicy, ActivityTask,
    ActivityType, ControlEvent, ControlEventKind, ControlLedger, ErrorCode, HotStateStore,
    IdempotencyKey, InMemoryControlLedger, InMemoryHotStateStore, RecoveryActionApplication,
    RecoveryAttempt, RecoveryItemScope, RecoveryLoopApplicationRequest, RecoveryPlanAction,
    RecoveryPolicy, RunId, RunRecoveryPlan, RunStatus, TaskQueue, TimerId, TimerRecord,
    TimerStatus, WorkerId, WorkerRef, apply_recovery_plan,
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

#[tokio::test]
async fn recovery_loop_requeues_run_scoped_activity_retry_after_attempt_record()
-> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-bounded-activity-retry-loop")?;
    let activity_id = ActivityId::new("activity-provider-retry-loop")?;
    let task_queue = TaskQueue::new("llm.openrouter")?;
    append_failed_provider_activity(&ledger, &run_id, &activity_id, &task_queue)?;
    let plan = RunRecoveryPlan {
        run_id: run_id.clone(),
        planned_at_ms: 100,
        actions: vec![RecoveryPlanAction::RetryActivity {
            scope: RecoveryItemScope::run(),
            activity_id: activity_id.clone(),
            retry_decision: ActivityRetryDecision::Retry {
                next_attempt: 2,
                backoff_ms: 50,
            },
        }],
    };

    let result = apply_recovery_plan(
        &ledger,
        &hot_state,
        RecoveryLoopApplicationRequest::new(plan, recovery_attempt(), 100, 7),
    )
    .await?;
    let events = ledger.load_events(&run_id)?;
    let view = ledger.load_run_view(&run_id)?;

    assert_eq!(result.attempt_record.sequence, 5);
    assert_eq!(result.action_results.len(), 1);
    assert!(matches!(
        result.action_results[0].result,
        RecoveryActionApplication::AppliedActivityRetry { .. }
    ));
    assert_eq!(events.len(), 5);
    assert!(matches!(
        events[4].event.kind,
        ControlEventKind::RecoveryStarted { .. }
    ));
    assert_eq!(view.status, RunStatus::Recovering);
    assert!(
        hot_state
            .claim_activity_task(worker_ref()?, Some(&task_queue), 149, 10)
            .await?
            .is_none(),
        "bounded recovery activity retry should respect backoff"
    );
    let lease = hot_state
        .claim_activity_task(worker_ref()?, Some(&task_queue), 150, 10)
        .await?
        .ok_or("missing requeued activity retry task")?;

    assert_eq!(lease.activity_task.task.run_id, run_id);
    assert_eq!(lease.activity_task.task.activity_id, activity_id);
    assert_eq!(lease.activity_task.task.next_attempt, 2);
    assert_eq!(lease.activity_task.priority, 7);
    assert_eq!(lease.activity_task.not_before_ms, 150);
    assert_eq!(
        lease.activity_task.metadata["recovery_action"],
        "retry_activity"
    );
    Ok(())
}

fn append_failed_provider_activity(
    ledger: &InMemoryControlLedger,
    run_id: &RunId,
    activity_id: &ActivityId,
    task_queue: &TaskQueue,
) -> Result<(), Box<dyn Error>> {
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        10,
        ControlEventKind::RunCreated {
            intent: "retry provider activity through bounded recovery loop".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        20,
        ControlEventKind::ActivityScheduled {
            task: ActivityTask::new(
                activity_id.clone(),
                ActivityType::new("llm.plan")?,
                task_queue.clone(),
                IdempotencyKey::new("run/activity/provider-retry-loop")?,
            )
            .with_retry_policy(ActivityRetryPolicy::new(3)?.with_initial_interval_ms(50)),
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        30,
        ControlEventKind::ActivityStarted {
            activity_id: activity_id.clone(),
            worker_id: Some(WorkerId::new("worker-openrouter")?),
            attempt: 1,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        40,
        ControlEventKind::ActivityFailed {
            activity_id: activity_id.clone(),
            failure: ActivityFailure {
                error_code: ErrorCode::new("provider_http_error")?,
                message: "provider returned HTTP 429".to_owned(),
                retryable: true,
                attempt: 1,
                metadata: serde_json::json!({"http_status": 429}),
            },
        },
    ))?;
    Ok(())
}

fn worker_ref() -> Result<WorkerRef, Box<dyn Error>> {
    Ok(WorkerRef {
        worker_id: WorkerId::new("worker-recovery-loop")?,
        capabilities: Vec::new(),
        metadata: serde_json::Value::Null,
    })
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
