//! Shared journal batch append and replay helpers.

use crate::{
    ControlError, ControlEvent, ControlEventRecord, ControlLedger, ControlResult, RunId, RunView,
};

/// Summary returned after recording one same-run control event batch.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlJournalBatchRecordingOutcome {
    /// Control-plane run id shared by all events in the batch.
    pub run_id: RunId,
    /// Number of events appended by this recording call.
    pub appended_event_count: usize,
    /// Ledger records returned by append operations.
    pub records: Vec<ControlEventRecord>,
    /// Ledger-replayed view after recording completed.
    pub run_view: RunView,
}

/// Records a same-run control event batch and replays the updated run view.
///
/// # Errors
///
/// Returns a control error when the batch is empty, when it contains events
/// from multiple run ids, or when the ledger rejects an append or replay.
pub fn record_control_event_batch<L>(
    ledger: &L,
    events: Vec<ControlEvent>,
) -> ControlResult<ControlJournalBatchRecordingOutcome>
where
    L: ControlLedger + ?Sized,
{
    let run_id = validate_batch_run_id(&events)?;
    let records = events
        .into_iter()
        .map(|event| ledger.append_event(event))
        .collect::<ControlResult<Vec<_>>>()?;
    let run_view = ledger.load_run_view(&run_id)?;
    Ok(ControlJournalBatchRecordingOutcome {
        run_id,
        appended_event_count: records.len(),
        records,
        run_view,
    })
}

fn validate_batch_run_id(events: &[ControlEvent]) -> ControlResult<RunId> {
    let Some(first) = events.first() else {
        return Err(ControlError::InvalidEventSequence {
            message: "control journal event batch cannot be empty".to_owned(),
        });
    };
    let run_id = first.run_id.clone();
    if events.iter().any(|event| event.run_id != run_id) {
        return Err(ControlError::InvalidEventSequence {
            message: "control journal event batch cannot mix run ids".to_owned(),
        });
    }
    Ok(run_id)
}
