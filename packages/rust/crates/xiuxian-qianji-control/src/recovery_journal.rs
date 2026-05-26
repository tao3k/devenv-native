//! Recovery journal recording helpers.

use crate::{
    ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    RecoveryAttempt, RecoveryItemScope, RunId,
};

/// Named request for recording one recovery attempt start.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecoveryStartedJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Run or step scope for the recovery attempt.
    pub scope: RecoveryItemScope,
    /// Recovery attempt payload.
    pub attempt: RecoveryAttempt,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
}

impl RecoveryStartedJournalRecord {
    /// Creates a recovery start journal record request.
    #[must_use]
    pub const fn new(
        run_id: RunId,
        scope: RecoveryItemScope,
        attempt: RecoveryAttempt,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            scope,
            attempt,
            occurred_at_ms,
        }
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let kind = ControlEventKind::RecoveryStarted {
            attempt: self.attempt,
        };
        match self.scope {
            RecoveryItemScope::Run => ControlEvent::run(self.run_id, self.occurred_at_ms, kind),
            RecoveryItemScope::Step { step_id } => {
                ControlEvent::step(self.run_id, step_id, self.occurred_at_ms, kind)
            }
        }
    }
}

/// Records one recovery start fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_recovery_started<L>(
    ledger: &L,
    request: RecoveryStartedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}
