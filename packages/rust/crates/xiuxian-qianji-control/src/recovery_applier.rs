//! Bounded recovery-action application helpers.

use crate::{
    ActivityRetryDecision, ActivityStatus, ActivityView, ControlEventRecord, ControlLedger,
    ControlResult, HotStateStore, RecoveryItemScope, RecoveryPlanAction, RunId,
    RunnableActivityTask, RunnableStep, StepId, StepLease, StepLeaseReleaseJournalRecord,
    StepQueueJournalRecord, TimerFireJournalRecord, WorkerActivityTask, record_step_lease_released,
    record_step_queued_with_hot_state, record_timer_fired,
};

/// Request for applying one recovery action.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecoveryActionApplicationRequest {
    /// Owning run id.
    pub run_id: RunId,
    /// Action to inspect and possibly apply.
    pub action: RecoveryPlanAction,
    /// Application timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Queue priority for applied retry steps.
    #[serde(default)]
    pub priority: i64,
}

impl RecoveryActionApplicationRequest {
    /// Creates a recovery action application request.
    #[must_use]
    pub const fn new(
        run_id: RunId,
        action: RecoveryPlanAction,
        occurred_at_ms: u64,
        priority: i64,
    ) -> Self {
        Self {
            run_id,
            action,
            occurred_at_ms,
            priority,
        }
    }
}

/// Result of applying or skipping one recovery action.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RecoveryActionApplication {
    /// A step-scoped retry was enqueued and recorded.
    AppliedStepRetry {
        /// Queued step.
        step: RunnableStep,
        /// Durable queue event.
        record: Box<ControlEventRecord>,
    },
    /// A run-scoped activity retry was requeued for worker polling.
    AppliedActivityRetry {
        /// Queued activity task.
        task: Box<RunnableActivityTask>,
    },
    /// A timer fire fact was recorded.
    AppliedTimerFire {
        /// Durable timer event.
        record: Box<ControlEventRecord>,
    },
    /// A hot lease was released and recorded.
    AppliedLeaseReclaim {
        /// Reclaimed lease.
        lease: StepLease,
        /// Durable lease release event.
        record: Box<ControlEventRecord>,
    },
    /// The action is not handled by this bounded applier.
    NotApplicable {
        /// Stable reason code.
        reason: RecoveryActionApplicationReason,
    },
}

/// Reason an action was not applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryActionApplicationReason {
    /// The recovery action kind is outside this applier's scope.
    UnsupportedAction,
    /// The retry action is run-scoped and cannot identify a step to enqueue.
    RunScopedRetry,
    /// Durable replay does not contain the requested activity.
    MissingReplayActivity,
    /// Durable replay contains the activity but not its original task payload.
    MissingReplayActivityTask,
    /// Durable replay contains the requested activity in a non-failed state.
    ActivityNotFailed,
    /// Durable replay does not contain a lease for the requested step.
    MissingReplayLease,
    /// Durable replay contains a different active lease than the action names.
    LeaseMismatch,
    /// Hot state no longer contains the replayed lease.
    HotLeaseMissing,
    /// The replayed lease is still active at the application timestamp.
    LeaseStillActive,
}

/// Applies one bounded recovery action.
///
/// # Errors
///
/// Returns a control error when hot-state enqueue or durable queue recording
/// fails. Unsupported actions return `NotApplicable` without side effects.
pub async fn apply_recovery_action<L, H>(
    ledger: &L,
    hot_state: &H,
    request: RecoveryActionApplicationRequest,
) -> ControlResult<RecoveryActionApplication>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let RecoveryActionApplicationRequest {
        run_id,
        action,
        occurred_at_ms,
        priority,
    } = request;
    match action {
        RecoveryPlanAction::RetryActivity {
            scope,
            activity_id,
            retry_decision:
                ActivityRetryDecision::Retry {
                    next_attempt,
                    backoff_ms,
                },
        } => {
            apply_retry_activity(
                ledger,
                hot_state,
                RetryActivityApplication {
                    run_id,
                    occurred_at_ms,
                    priority,
                    scope,
                    activity_id,
                    next_attempt,
                    backoff_ms,
                },
            )
            .await
        }
        RecoveryPlanAction::FireTimer {
            scope,
            timer_id,
            fire_at_ms,
        } => {
            let event_time_ms = fire_at_ms.unwrap_or(occurred_at_ms);
            let record = record_timer_fired(
                ledger,
                TimerFireJournalRecord::new(run_id, scope, timer_id, event_time_ms),
            )?;
            Ok(RecoveryActionApplication::AppliedTimerFire {
                record: Box::new(record),
            })
        }
        RecoveryPlanAction::ReclaimExpiredLease { step_id, lease_id } => {
            let Some(lease) = replayed_step_lease(ledger, &run_id, &step_id)? else {
                return Ok(RecoveryActionApplication::NotApplicable {
                    reason: RecoveryActionApplicationReason::MissingReplayLease,
                });
            };
            if lease.lease_id != lease_id {
                return Ok(RecoveryActionApplication::NotApplicable {
                    reason: RecoveryActionApplicationReason::LeaseMismatch,
                });
            }
            if lease.is_active_at(occurred_at_ms) {
                return Ok(RecoveryActionApplication::NotApplicable {
                    reason: RecoveryActionApplicationReason::LeaseStillActive,
                });
            }
            if !hot_state
                .reclaim_expired_lease(&lease, occurred_at_ms)
                .await?
            {
                return Ok(RecoveryActionApplication::NotApplicable {
                    reason: RecoveryActionApplicationReason::HotLeaseMissing,
                });
            }
            let record = record_step_lease_released(
                ledger,
                StepLeaseReleaseJournalRecord::new(lease.clone(), occurred_at_ms),
            )?;
            Ok(RecoveryActionApplication::AppliedLeaseReclaim {
                lease,
                record: Box::new(record),
            })
        }
        _ => Ok(RecoveryActionApplication::NotApplicable {
            reason: RecoveryActionApplicationReason::UnsupportedAction,
        }),
    }
}

struct RetryActivityApplication {
    run_id: RunId,
    occurred_at_ms: u64,
    priority: i64,
    scope: RecoveryItemScope,
    activity_id: crate::ActivityId,
    next_attempt: u32,
    backoff_ms: u64,
}

async fn apply_retry_activity<L, H>(
    ledger: &L,
    hot_state: &H,
    request: RetryActivityApplication,
) -> ControlResult<RecoveryActionApplication>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let RetryActivityApplication {
        run_id,
        occurred_at_ms,
        priority,
        scope,
        activity_id,
        next_attempt,
        backoff_ms,
    } = request;
    if matches!(scope, RecoveryItemScope::Run) {
        return apply_activity_retry(
            ledger,
            hot_state,
            ActivityRetryApplication {
                run_id,
                occurred_at_ms,
                priority,
                scope,
                activity_id,
                next_attempt,
                backoff_ms,
            },
        )
        .await;
    }
    apply_step_retry(
        ledger,
        hot_state,
        StepRetryApplication {
            run_id,
            occurred_at_ms,
            priority,
            scope,
            activity_id,
            next_attempt,
            backoff_ms,
        },
    )
    .await
}

fn replayed_step_lease<L>(
    ledger: &L,
    run_id: &RunId,
    step_id: &StepId,
) -> ControlResult<Option<StepLease>>
where
    L: ControlLedger + ?Sized,
{
    Ok(ledger
        .load_run_view(run_id)?
        .steps
        .get(step_id)
        .and_then(|step| step.active_lease.clone()))
}

struct ActivityRetryApplication {
    run_id: RunId,
    occurred_at_ms: u64,
    priority: i64,
    scope: RecoveryItemScope,
    activity_id: crate::ActivityId,
    next_attempt: u32,
    backoff_ms: u64,
}

async fn apply_activity_retry<L, H>(
    ledger: &L,
    hot_state: &H,
    request: ActivityRetryApplication,
) -> ControlResult<RecoveryActionApplication>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let ActivityRetryApplication {
        run_id,
        occurred_at_ms,
        priority,
        scope,
        activity_id,
        next_attempt,
        backoff_ms,
    } = request;
    let Some(activity) = replayed_activity_for_scope(ledger, &run_id, &scope, &activity_id)? else {
        return Ok(RecoveryActionApplication::NotApplicable {
            reason: RecoveryActionApplicationReason::MissingReplayActivity,
        });
    };
    if activity.status != ActivityStatus::Failed {
        return Ok(RecoveryActionApplication::NotApplicable {
            reason: RecoveryActionApplicationReason::ActivityNotFailed,
        });
    }
    let Some(task) = activity.task else {
        return Ok(RecoveryActionApplication::NotApplicable {
            reason: RecoveryActionApplicationReason::MissingReplayActivityTask,
        });
    };
    let runnable = RunnableActivityTask {
        task: WorkerActivityTask {
            run_id: run_id.clone(),
            step_id: step_id_for_scope(&scope),
            activity_id: task.activity_id,
            activity_type: task.activity_type,
            task_queue: task.task_queue,
            next_attempt,
            scheduled_at_ms: occurred_at_ms,
            input_ref: task.input_ref,
            idempotency_key: task.idempotency_key,
            retry_policy: task.retry_policy,
            timeout_ms: task.timeout_ms,
            metadata: task.metadata,
        },
        priority,
        not_before_ms: occurred_at_ms.saturating_add(backoff_ms),
        metadata: serde_json::json!({
            "recovery_action": "retry_activity",
            "activity_id": activity_id.as_str(),
            "next_attempt": next_attempt,
        }),
    };
    hot_state.enqueue_activity_task(runnable.clone()).await?;
    Ok(RecoveryActionApplication::AppliedActivityRetry {
        task: Box::new(runnable),
    })
}

fn replayed_activity_for_scope<L>(
    ledger: &L,
    run_id: &RunId,
    scope: &RecoveryItemScope,
    activity_id: &crate::ActivityId,
) -> ControlResult<Option<ActivityView>>
where
    L: ControlLedger + ?Sized,
{
    let view = ledger.load_run_view(run_id)?;
    Ok(match scope {
        RecoveryItemScope::Run => view.activities.get(activity_id).cloned(),
        RecoveryItemScope::Step { step_id } => view
            .steps
            .get(step_id)
            .and_then(|step| step.activities.get(activity_id))
            .cloned(),
    })
}

fn step_id_for_scope(scope: &RecoveryItemScope) -> Option<StepId> {
    match scope {
        RecoveryItemScope::Run => None,
        RecoveryItemScope::Step { step_id } => Some(step_id.clone()),
    }
}

struct StepRetryApplication {
    run_id: RunId,
    occurred_at_ms: u64,
    priority: i64,
    scope: RecoveryItemScope,
    activity_id: crate::ActivityId,
    next_attempt: u32,
    backoff_ms: u64,
}

async fn apply_step_retry<L, H>(
    ledger: &L,
    hot_state: &H,
    request: StepRetryApplication,
) -> ControlResult<RecoveryActionApplication>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let StepRetryApplication {
        run_id,
        occurred_at_ms,
        priority,
        scope,
        activity_id,
        next_attempt,
        backoff_ms,
    } = request;
    let RecoveryItemScope::Step { step_id } = scope else {
        return Ok(RecoveryActionApplication::NotApplicable {
            reason: RecoveryActionApplicationReason::RunScopedRetry,
        });
    };
    let step = RunnableStep {
        run_id,
        step_id,
        priority,
        not_before_ms: occurred_at_ms.saturating_add(backoff_ms),
        metadata: serde_json::json!({
            "recovery_action": "retry_activity",
            "activity_id": activity_id.as_str(),
            "next_attempt": next_attempt,
        }),
    };
    let record = record_step_queued_with_hot_state(
        ledger,
        hot_state,
        StepQueueJournalRecord::new(step.clone(), occurred_at_ms),
    )
    .await?;
    Ok(RecoveryActionApplication::AppliedStepRetry {
        step,
        record: Box::new(record),
    })
}
