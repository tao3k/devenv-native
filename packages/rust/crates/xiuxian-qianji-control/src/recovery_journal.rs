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
    let RecoveryStartedJournalRecord {
        run_id,
        scope,
        attempt,
        occurred_at_ms,
    } = request;
    let kind = ControlEventKind::RecoveryStarted { attempt };
    let event = match scope {
        RecoveryItemScope::Run => ControlEvent::run(run_id, occurred_at_ms, kind),
        RecoveryItemScope::Step { step_id } => {
            ControlEvent::step(run_id, step_id, occurred_at_ms, kind)
        }
    };
    ledger.append_event(event)
}
