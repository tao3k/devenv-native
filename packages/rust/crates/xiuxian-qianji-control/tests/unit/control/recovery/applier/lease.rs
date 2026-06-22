use std::error::Error;

use super::support::worker_ref;
use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, HotStateStore, InMemoryControlLedger,
    InMemoryHotStateStore, LeaseId, RecoveryActionApplication, RecoveryActionApplicationReason,
    RecoveryActionApplicationRequest, RecoveryPlanAction, RunId, RunnableStep, StepId, StepStatus,
    apply_recovery_action,
};

#[tokio::test]
async fn recovery_lease_applier_reclaims_expired_hot_lease() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-recovery-reclaim-lease")?;
    let step_id = StepId::new("stage-expired-lease")?;
    hot_state
        .enqueue_step(RunnableStep {
            run_id: run_id.clone(),
            step_id: step_id.clone(),
            priority: 4,
            not_before_ms: 0,
            metadata: serde_json::Value::Null,
        })
        .await?;
    let lease = hot_state
        .acquire_lease(worker_ref()?, 100, 10)
        .await?
        .ok_or("missing acquired lease")?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        100,
        ControlEventKind::StepLeaseAcquired {
            lease: lease.clone(),
        },
    ))?;
    let action = RecoveryPlanAction::ReclaimExpiredLease {
        step_id: step_id.clone(),
        lease_id: lease.lease_id.clone(),
    };

    let result = apply_recovery_action(
        &ledger,
        &hot_state,
        RecoveryActionApplicationRequest::new(run_id.clone(), action, 120, 0),
    )
    .await?;

    assert!(matches!(
        result,
        RecoveryActionApplication::AppliedLeaseReclaim { .. }
    ));
    let view = ledger.load_run_view(&run_id)?;
    let step = view.steps.get(&step_id).ok_or("missing step view")?;
    assert_eq!(step.status, StepStatus::Queued);
    assert!(step.active_lease.is_none());
    assert!(
        hot_state
            .acquire_lease(worker_ref()?, 120, 10)
            .await?
            .is_some(),
        "expired lease reclaim should requeue the step"
    );
    Ok(())
}

#[tokio::test]
async fn recovery_lease_applier_rejects_still_active_lease_without_side_effects()
-> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-recovery-active-lease")?;
    let step_id = StepId::new("stage-active-lease")?;
    hot_state
        .enqueue_step(RunnableStep {
            run_id: run_id.clone(),
            step_id: step_id.clone(),
            priority: 4,
            not_before_ms: 0,
            metadata: serde_json::Value::Null,
        })
        .await?;
    let lease = hot_state
        .acquire_lease(worker_ref()?, 100, 1_000)
        .await?
        .ok_or("missing acquired lease")?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        100,
        ControlEventKind::StepLeaseAcquired { lease },
    ))?;
    let action = RecoveryPlanAction::ReclaimExpiredLease {
        step_id,
        lease_id: LeaseId::new("lease-1")?,
    };

    let result = apply_recovery_action(
        &ledger,
        &hot_state,
        RecoveryActionApplicationRequest::new(run_id.clone(), action, 120, 0),
    )
    .await?;

    assert_eq!(
        result,
        RecoveryActionApplication::NotApplicable {
            reason: RecoveryActionApplicationReason::LeaseStillActive,
        }
    );
    assert_eq!(ledger.load_events(&run_id)?.len(), 1);
    assert!(
        hot_state
            .acquire_lease(worker_ref()?, 120, 10)
            .await?
            .is_none(),
        "still-active lease should remain leased"
    );
    Ok(())
}
