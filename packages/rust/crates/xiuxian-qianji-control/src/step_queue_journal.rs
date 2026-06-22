//! Step queue journal recording helpers.

use crate::{
    ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    HotStateStore, RunnableStep,
};

/// Named request for recording one queued step fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StepQueueJournalRecord {
    /// Runnable step to record.
    pub step: RunnableStep,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
}

impl StepQueueJournalRecord {
    /// Creates a step queue journal record request.
    #[must_use]
    pub const fn new(step: RunnableStep, occurred_at_ms: u64) -> Self {
        Self {
            step,
            occurred_at_ms,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            step,
            occurred_at_ms,
        } = self;
        ControlEvent::step(
            step.run_id,
            step.step_id,
            occurred_at_ms,
            ControlEventKind::StepQueued,
        )
    }
}

/// Records one step queue fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_step_queued<L>(
    ledger: &L,
    request: StepQueueJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}

/// Enqueues one runnable step in hot state and records the durable queue fact.
///
/// # Errors
///
/// Returns a control error when hot-state enqueue fails or durable ledger
/// append fails. The hot-state write happens first so durable `StepQueued`
/// history represents a successful queue mirror.
pub async fn record_step_queued_with_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: StepQueueJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    hot_state.enqueue_step(request.step.clone()).await?;
    record_step_queued(ledger, request)
}
