//! Step lease journal recording helpers.

use crate::{
    ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult, StepLease,
};

/// Named request for recording one released lease fact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StepLeaseReleaseJournalRecord {
    /// Released lease.
    pub lease: StepLease,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
}

impl StepLeaseReleaseJournalRecord {
    /// Creates a step lease release journal record request.
    #[must_use]
    pub const fn new(lease: StepLease, occurred_at_ms: u64) -> Self {
        Self {
            lease,
            occurred_at_ms,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            lease,
            occurred_at_ms,
        } = self;
        ControlEvent::step(
            lease.run_id.clone(),
            lease.step_id.clone(),
            occurred_at_ms,
            ControlEventKind::StepLeaseReleased { lease },
        )
    }
}

/// Records one step lease release fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_step_lease_released<L>(
    ledger: &L,
    request: StepLeaseReleaseJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}
