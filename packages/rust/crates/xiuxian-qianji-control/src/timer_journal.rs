//! Durable timer journal recording helpers.

use crate::{
    ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    RecoveryItemScope, RunId, TimerId,
};

/// Named request for recording one fired timer fact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TimerFireJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Run or step scope for the timer.
    pub scope: RecoveryItemScope,
    /// Timer id to mark fired.
    pub timer_id: TimerId,
    /// Event timestamp supplied by caller or the recovery plan.
    pub occurred_at_ms: u64,
}

impl TimerFireJournalRecord {
    /// Creates a timer fire journal record request.
    #[must_use]
    pub const fn new(
        run_id: RunId,
        scope: RecoveryItemScope,
        timer_id: TimerId,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            scope,
            timer_id,
            occurred_at_ms,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            run_id,
            scope,
            timer_id,
            occurred_at_ms,
        } = self;
        let kind = ControlEventKind::TimerFired { timer_id };
        match scope {
            RecoveryItemScope::Run => ControlEvent::run(run_id, occurred_at_ms, kind),
            RecoveryItemScope::Step { step_id } => {
                ControlEvent::step(run_id, step_id, occurred_at_ms, kind)
            }
        }
    }
}

/// Records one timer fire fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_timer_fired<L>(
    ledger: &L,
    request: TimerFireJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}
