use std::error::Error;

use super::support::worker_ref;
use crate::control::recovery::support::{activity_failure, append_run_created};
use xiuxian_qianji_control::{
    ActivityId, ActivityRetryDecision, ActivityRetryPolicy, ActivityTask, ActivityType,
    ControlEvent, ControlEventKind, ControlLedger, HotStateStore, IdempotencyKey,
    InMemoryControlLedger, InMemoryHotStateStore, RecoveryActionApplication,
    RecoveryActionApplicationReason, RecoveryActionApplicationRequest, RecoveryItemScope,
    RecoveryPlanAction, RunId, StepId, StepStatus, TaskQueue, WorkerId, apply_recovery_action,
};

#[tokio::test]
async fn recovery_retry_applier_enqueues_step_after_backoff() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-recovery-retry-applier")?;
    let step_id = StepId::new("stage-retry")?;
    let activity_id = ActivityId::new("activity-retry")?;
    let action = RecoveryPlanAction::RetryActivity {
        scope: RecoveryItemScope::step(step_id.clone()),
        activity_id: activity_id.clone(),
        retry_decision: ActivityRetryDecision::Retry {
            next_attempt: 3,
            backoff_ms: 25,
        },
    };

    let result = apply_recovery_action(
        &ledger,
        &hot_state,
        RecoveryActionApplicationRequest::new(run_id.clone(), action, 100, 9),
    )
    .await?;
    assert!(matches!(
        result,
        RecoveryActionApplication::AppliedStepRetry { .. }
    ));
    assert!(
        hot_state
            .acquire_lease(worker_ref()?, 124, 10)
            .await?
            .is_none(),
        "retry step should respect backoff"
    );
    let lease = hot_state
        .acquire_lease(worker_ref()?, 125, 10)
        .await?
        .ok_or("missing retry lease")?;
    let view = ledger.load_run_view(&run_id)?;
    let step_view = view.steps.get(&step_id).ok_or("missing queued step")?;

    assert_eq!(lease.step_id, step_id);
    assert_eq!(step_view.status, StepStatus::Queued);
    assert_eq!(ledger.load_events(&run_id)?.len(), 1);
    if let RecoveryActionApplication::AppliedStepRetry { step, .. } = result {
        assert_eq!(step.metadata["activity_id"], activity_id.as_str());
        assert_eq!(step.metadata["next_attempt"], 3);
    }
    Ok(())
}

#[tokio::test]
async fn recovery_retry_applier_rejects_run_scoped_retry_without_side_effects()
-> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-recovery-run-scope-retry")?;
    append_run_created(&ledger, run_id.clone())?;
    let action = RecoveryPlanAction::RetryActivity {
        scope: RecoveryItemScope::run(),
        activity_id: ActivityId::new("activity-run-retry")?,
        retry_decision: ActivityRetryDecision::Retry {
            next_attempt: 2,
            backoff_ms: 10,
        },
    };

    let result = apply_recovery_action(
        &ledger,
        &hot_state,
        RecoveryActionApplicationRequest::new(run_id.clone(), action, 100, 9),
    )
    .await?;

    assert_eq!(
        result,
        RecoveryActionApplication::NotApplicable {
            reason: RecoveryActionApplicationReason::MissingReplayActivity,
        }
    );
    assert_eq!(ledger.load_events(&run_id)?.len(), 1);
    assert!(
        hot_state
            .acquire_lease(worker_ref()?, 100, 10)
            .await?
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn recovery_retry_applier_requeues_run_scoped_activity_after_backoff()
-> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-recovery-activity-retry")?;
    let activity_id = ActivityId::new("activity-llm-provider-retry")?;
    let task_queue = TaskQueue::new("llm.openrouter")?;
    append_run_created(&ledger, run_id.clone())?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        10,
        ControlEventKind::ActivityScheduled {
            task: ActivityTask::new(
                activity_id.clone(),
                ActivityType::new("llm.plan")?,
                task_queue.clone(),
                IdempotencyKey::new("run/activity/llm-provider-retry")?,
            )
            .with_retry_policy(ActivityRetryPolicy::new(3)?.with_initial_interval_ms(25)),
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        20,
        ControlEventKind::ActivityStarted {
            activity_id: activity_id.clone(),
            worker_id: Some(WorkerId::new("worker-openrouter")?),
            attempt: 1,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        30,
        ControlEventKind::ActivityFailed {
            activity_id: activity_id.clone(),
            failure: activity_failure("provider_http_error", true, 1)?,
        },
    ))?;
    let action = RecoveryPlanAction::RetryActivity {
        scope: RecoveryItemScope::run(),
        activity_id: activity_id.clone(),
        retry_decision: ActivityRetryDecision::Retry {
            next_attempt: 2,
            backoff_ms: 25,
        },
    };

    let result = apply_recovery_action(
        &ledger,
        &hot_state,
        RecoveryActionApplicationRequest::new(run_id.clone(), action, 100, 9),
    )
    .await?;

    assert!(matches!(
        result,
        RecoveryActionApplication::AppliedActivityRetry { .. }
    ));
    assert!(
        hot_state
            .claim_activity_task(worker_ref()?, Some(&task_queue), 124, 10)
            .await?
            .is_none(),
        "activity retry should respect backoff"
    );
    let lease = hot_state
        .claim_activity_task(worker_ref()?, Some(&task_queue), 125, 10)
        .await?
        .ok_or("missing activity retry lease")?;

    assert_eq!(lease.activity_task.task.run_id, run_id);
    assert_eq!(lease.activity_task.task.activity_id, activity_id);
    assert_eq!(lease.activity_task.task.step_id, None);
    assert_eq!(lease.activity_task.task.next_attempt, 2);
    assert_eq!(lease.activity_task.task.task_queue, task_queue);
    assert_eq!(lease.activity_task.priority, 9);
    assert_eq!(lease.activity_task.not_before_ms, 125);
    assert_eq!(
        lease.activity_task.metadata["recovery_action"],
        "retry_activity"
    );
    assert_eq!(ledger.load_events(&run_id)?.len(), 4);
    if let RecoveryActionApplication::AppliedActivityRetry { task } = result {
        assert_eq!(task.task.next_attempt, 2);
        assert_eq!(task.metadata["activity_id"], "activity-llm-provider-retry");
    }
    Ok(())
}

#[tokio::test]
async fn recovery_retry_applier_skips_non_retry_actions_without_side_effects()
-> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-recovery-non-retry")?;
    let action = RecoveryPlanAction::AwaitHumanInput {
        step_id: StepId::new("stage-wait")?,
    };

    let result = apply_recovery_action(
        &ledger,
        &hot_state,
        RecoveryActionApplicationRequest::new(run_id.clone(), action, 100, 9),
    )
    .await?;

    assert_eq!(
        result,
        RecoveryActionApplication::NotApplicable {
            reason: RecoveryActionApplicationReason::UnsupportedAction,
        }
    );
    assert!(ledger.load_events(&run_id)?.is_empty());
    assert!(
        hot_state
            .acquire_lease(worker_ref()?, 100, 10)
            .await?
            .is_none()
    );
    Ok(())
}
