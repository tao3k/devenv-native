//! Durable signal journal recording helpers.

use crate::{
    ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    RecoveryItemScope, RunId, SignalRecord,
};

/// Named request for recording one received signal fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignalReceiveJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Run or step scope for the signal.
    pub scope: RecoveryItemScope,
    /// Signal payload and metadata.
    pub signal: SignalRecord,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl SignalReceiveJournalRecord {
    /// Creates a signal receive journal record request.
    #[must_use]
    pub const fn new(
        run_id: RunId,
        scope: RecoveryItemScope,
        signal: SignalRecord,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            scope,
            signal,
            occurred_at_ms,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            run_id,
            scope,
            signal,
            occurred_at_ms,
        } = self;
        let kind = ControlEventKind::SignalReceived { signal };
        match scope {
            RecoveryItemScope::Run => ControlEvent::run(run_id, occurred_at_ms, kind),
            RecoveryItemScope::Step { step_id } => {
                ControlEvent::step(run_id, step_id, occurred_at_ms, kind)
            }
        }
    }
}

/// Records one signal received fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_signal_received<L>(
    ledger: &L,
    request: SignalReceiveJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}
