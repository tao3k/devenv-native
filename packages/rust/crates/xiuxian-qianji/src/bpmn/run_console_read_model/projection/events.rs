//! Control-event row projection.

use super::{
    QianjiRunConsoleEventRow,
    event_text::{event_kind_name, event_message},
};
use num_traits::ToPrimitive;
use xiuxian_qianji_control::{ControlEventRecord, RunId};

/// Project control events into qianji run-console event rows.
///
/// # Errors
///
/// Returns an error when a ledger sequence exceeds the Arrow `Int32` range
/// used by the JavaScript read-model contract.
pub(crate) fn qianji_run_console_event_rows(
    run_id: &RunId,
    events: &[ControlEventRecord],
) -> Result<Vec<QianjiRunConsoleEventRow>, String> {
    events
        .iter()
        .map(|record| event_row_from_record(run_id, record))
        .collect()
}

fn event_row_from_record(
    run_id: &RunId,
    record: &ControlEventRecord,
) -> Result<QianjiRunConsoleEventRow, String> {
    let sequence = i32::try_from(record.sequence).map_err(|_| {
        format!(
            "qianji run-console event sequence {} exceeds Int32 range",
            record.sequence
        )
    })?;
    Ok(QianjiRunConsoleEventRow {
        run_id: run_id.as_str().to_owned(),
        event_id: record.sequence.to_string(),
        sequence,
        kind: event_kind_name(&record.event.kind).to_owned(),
        message: event_message(&record.event.kind),
        step_id: record
            .event
            .step_id
            .as_ref()
            .map(|step_id| step_id.as_str().to_owned()),
        occurred_at_ms: record
            .event
            .occurred_at_ms
            .to_f64()
            .ok_or_else(|| "event timestamp cannot be represented as Float64".to_string())?,
    })
}
