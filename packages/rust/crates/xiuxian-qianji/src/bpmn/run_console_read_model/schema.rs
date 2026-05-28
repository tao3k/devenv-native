//! Arrow schema and `RecordBatch` builders for qianji run-console rows.

use super::projection::{
    QianjiRunConsoleElementStateRow, QianjiRunConsoleEventRow,
    qianji_run_console_element_state_rows, qianji_run_console_event_rows,
};
use arrow::{
    array::{ArrayRef, Float64Array, Int32Array, StringArray},
    error::ArrowError,
    record_batch::RecordBatch,
};
use std::{collections::HashMap, sync::Arc};
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, WENDAO_TABLE_METADATA_KEY,
    build_arrow_schema, validate_record_batch_schema,
};
use xiuxian_qianji_control::{ControlEventRecord, RunId};

const EVENT_TABLE: &str = "qianji.run_console.event.v1";
const ELEMENT_STATE_TABLE: &str = "qianji.run_console.element_state.v1";

/// Arrow read model containing qianji run-console tables.
#[derive(Debug, Clone)]
pub struct QianjiRunConsoleArrowReadModel {
    /// Control-event row batch.
    pub events: RecordBatch,
    /// BPMN element-state row batch.
    pub element_states: RecordBatch,
}

/// Build the qianji run-console Arrow read model from control events.
///
/// # Errors
///
/// Returns an Arrow error when schema construction or row projection fails.
pub fn qianji_run_console_arrow_read_model(
    run_id: &RunId,
    events: &[ControlEventRecord],
) -> Result<QianjiRunConsoleArrowReadModel, ArrowError> {
    let event_rows =
        qianji_run_console_event_rows(run_id, events).map_err(ArrowError::SchemaError)?;
    let element_state_rows = qianji_run_console_element_state_rows(run_id, events);
    Ok(QianjiRunConsoleArrowReadModel {
        events: qianji_run_console_event_record_batch(&event_rows)?,
        element_states: qianji_run_console_element_state_record_batch(&element_state_rows)?,
    })
}

/// Return the logical Arrow contract for qianji run-console event rows.
#[must_use]
pub fn qianji_run_console_event_arrow_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        EVENT_TABLE,
        true,
        vec![
            ArrowSchemaColumn::new("runId", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::new("eventId", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::new("sequence", ArrowSchemaDataType::Int32),
            ArrowSchemaColumn::new("kind", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::new("message", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::nullable("stepId", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::new("occurredAtMs", ArrowSchemaDataType::Float64),
        ],
    )
}

/// Return the Arrow schema for qianji run-console event rows.
#[must_use]
pub fn qianji_run_console_event_arrow_schema() -> Arc<arrow::datatypes::Schema> {
    Arc::new(build_arrow_schema(
        &qianji_run_console_event_arrow_contract(),
        table_metadata(EVENT_TABLE),
    ))
}

/// Return the logical Arrow contract for qianji run-console element-state rows.
#[must_use]
pub fn qianji_run_console_element_state_arrow_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        ELEMENT_STATE_TABLE,
        true,
        vec![
            ArrowSchemaColumn::new("runId", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::new("elementId", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::new("state", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::new("sourceEventId", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::new("message", ArrowSchemaDataType::Utf8),
        ],
    )
}

/// Return the Arrow schema for qianji run-console element-state rows.
#[must_use]
pub fn qianji_run_console_element_state_arrow_schema() -> Arc<arrow::datatypes::Schema> {
    Arc::new(build_arrow_schema(
        &qianji_run_console_element_state_arrow_contract(),
        table_metadata(ELEMENT_STATE_TABLE),
    ))
}

fn qianji_run_console_event_record_batch(
    rows: &[QianjiRunConsoleEventRow],
) -> Result<RecordBatch, ArrowError> {
    let batch = RecordBatch::try_new(
        qianji_run_console_event_arrow_schema(),
        vec![
            string_array(rows.iter().map(|row| Some(row.run_id.as_str()))),
            string_array(rows.iter().map(|row| Some(row.event_id.as_str()))),
            Arc::new(Int32Array::from(
                rows.iter().map(|row| row.sequence).collect::<Vec<_>>(),
            )),
            string_array(rows.iter().map(|row| Some(row.kind.as_str()))),
            string_array(rows.iter().map(|row| Some(row.message.as_str()))),
            string_array(rows.iter().map(|row| row.step_id.as_deref())),
            Arc::new(Float64Array::from(
                rows.iter()
                    .map(|row| row.occurred_at_ms)
                    .collect::<Vec<_>>(),
            )),
        ],
    )?;
    validate_record_batch_schema(&batch, &qianji_run_console_event_arrow_contract())
        .map_err(|error| ArrowError::SchemaError(error.to_string()))?;
    Ok(batch)
}

fn qianji_run_console_element_state_record_batch(
    rows: &[QianjiRunConsoleElementStateRow],
) -> Result<RecordBatch, ArrowError> {
    let batch = RecordBatch::try_new(
        qianji_run_console_element_state_arrow_schema(),
        vec![
            string_array(rows.iter().map(|row| Some(row.run_id.as_str()))),
            string_array(rows.iter().map(|row| Some(row.element_id.as_str()))),
            string_array(rows.iter().map(|row| Some(row.state.as_str()))),
            string_array(rows.iter().map(|row| Some(row.source_event_id.as_str()))),
            string_array(rows.iter().map(|row| Some(row.message.as_str()))),
        ],
    )?;
    validate_record_batch_schema(&batch, &qianji_run_console_element_state_arrow_contract())
        .map_err(|error| ArrowError::SchemaError(error.to_string()))?;
    Ok(batch)
}

fn string_array<'a>(values: impl Iterator<Item = Option<&'a str>>) -> ArrayRef {
    Arc::new(StringArray::from(values.collect::<Vec<_>>()))
}

fn table_metadata(table: &str) -> HashMap<String, String> {
    HashMap::from([(WENDAO_TABLE_METADATA_KEY.to_owned(), table.to_owned())])
}
