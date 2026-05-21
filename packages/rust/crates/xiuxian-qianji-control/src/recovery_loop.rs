//! Bounded recovery-loop application helpers.

use crate::{
    ControlEventRecord, ControlLedger, ControlResult, HotStateStore, RecoveryActionApplication,
    RecoveryActionApplicationRequest, RecoveryAttempt, RecoveryItemScope, RecoveryPlanAction,
    RecoveryStartedJournalRecord, RunRecoveryPlan, apply_recovery_action, record_recovery_started,
};

/// Request for applying a bounded recovery plan.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecoveryLoopApplicationRequest {
    /// Plan whose ordered actions should be applied.
    pub plan: RunRecoveryPlan,
    /// Recovery attempt to record before action application.
    pub attempt: RecoveryAttempt,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Queue priority for applied retry steps.
    #[serde(default)]
    pub priority: i64,
}

impl RecoveryLoopApplicationRequest {
    /// Creates a bounded recovery loop application request.
    #[must_use]
    pub const fn new(
        plan: RunRecoveryPlan,
        attempt: RecoveryAttempt,
        occurred_at_ms: u64,
        priority: i64,
    ) -> Self {
        Self {
            plan,
            attempt,
            occurred_at_ms,
            priority,
        }
    }
}

/// Result of applying a bounded recovery plan.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecoveryLoopApplication {
    /// Durable recovery-start event.
    pub attempt_record: ControlEventRecord,
    /// Per-action application trace, in plan order.
    #[serde(default)]
    pub action_results: Vec<RecoveryLoopActionApplication>,
}

/// Per-action application trace.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecoveryLoopActionApplication {
    /// Action that was applied or skipped.
    pub action: RecoveryPlanAction,
    /// Action application result.
    pub result: RecoveryActionApplication,
}

/// Records one recovery attempt and applies the supplied plan in order.
///
/// # Errors
///
/// Returns a control error when recording the recovery attempt or applying an
/// executable recovery action fails.
pub async fn apply_recovery_plan<L, H>(
    ledger: &L,
    hot_state: &H,
    request: RecoveryLoopApplicationRequest,
) -> ControlResult<RecoveryLoopApplication>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let RecoveryLoopApplicationRequest {
        plan,
        attempt,
        occurred_at_ms,
        priority,
    } = request;
    let attempt_record = record_recovery_started(
        ledger,
        RecoveryStartedJournalRecord::new(
            plan.run_id.clone(),
            RecoveryItemScope::run(),
            attempt,
            occurred_at_ms,
        ),
    )?;
    let mut action_results = Vec::with_capacity(plan.actions.len());
    for action in plan.actions {
        let result = apply_recovery_action(
            ledger,
            hot_state,
            RecoveryActionApplicationRequest::new(
                plan.run_id.clone(),
                action.clone(),
                occurred_at_ms,
                priority,
            ),
        )
        .await?;
        action_results.push(RecoveryLoopActionApplication { action, result });
    }
    Ok(RecoveryLoopApplication {
        attempt_record,
        action_results,
    })
}
