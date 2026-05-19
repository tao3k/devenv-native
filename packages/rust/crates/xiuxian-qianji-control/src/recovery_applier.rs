//! Bounded recovery-action application helpers.

use crate::{
    ActivityRetryDecision, ControlEventRecord, ControlLedger, ControlResult, HotStateStore,
    RecoveryItemScope, RecoveryPlanAction, RunId, RunnableStep, StepQueueJournalRecord,
    TimerFireJournalRecord, record_step_queued_with_hot_state, record_timer_fired,
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
    /// A timer fire fact was recorded.
    AppliedTimerFire {
        /// Durable timer event.
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
        _ => Ok(RecoveryActionApplication::NotApplicable {
            reason: RecoveryActionApplicationReason::UnsupportedAction,
        }),
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
