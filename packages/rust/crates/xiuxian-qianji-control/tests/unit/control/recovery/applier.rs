use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityRetryDecision, ControlLedger, HotStateStore, InMemoryControlLedger,
    InMemoryHotStateStore, RecoveryActionApplication, RecoveryActionApplicationReason,
    RecoveryActionApplicationRequest, RecoveryItemScope, RecoveryPlanAction, RunId, StepId,
    StepStatus, WorkerId, WorkerRef, apply_recovery_action,
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
            reason: RecoveryActionApplicationReason::RunScopedRetry,
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

fn worker_ref() -> Result<WorkerRef, Box<dyn Error>> {
    Ok(WorkerRef {
        worker_id: WorkerId::new("worker-recovery-applier")?,
        capabilities: Vec::new(),
        metadata: serde_json::Value::Null,
    })
}
