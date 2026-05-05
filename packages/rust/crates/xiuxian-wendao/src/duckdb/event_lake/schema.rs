//! Wendao event-lake table and `Arrow` schema contracts.

use std::sync::{Arc, OnceLock};

use arrow::array::{ArrayRef, StringArray, TimestampMillisecondArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::duckdb::{ensure_duckdb_identifier, quoted_duckdb_identifier};

use super::record::WendaoEventRecord;

/// Wendao-owned event table name inside an attached DuckLake catalog.
pub const WENDAO_EVENT_LAKE_EVENTS_TABLE: &str = "events";

pub(crate) const TENANT_ID_COLUMN: &str = "tenant_id";
pub(crate) const CASE_ID_COLUMN: &str = "case_id";
pub(crate) const EVENT_TYPE_COLUMN: &str = "event_type";
pub(crate) const PAYLOAD_COLUMN: &str = "payload";
pub(crate) const CREATED_AT_COLUMN: &str = "created_at";

/// Return the Arrow schema used for Wendao event-lake appends.
#[must_use]
pub fn wendao_event_schema() -> SchemaRef {
    static SCHEMA: OnceLock<SchemaRef> = OnceLock::new();
    Arc::clone(SCHEMA.get_or_init(|| {
        Arc::new(Schema::new(vec![
            Field::new(TENANT_ID_COLUMN, DataType::Utf8, false),
            Field::new(CASE_ID_COLUMN, DataType::Utf8, false),
            Field::new(EVENT_TYPE_COLUMN, DataType::Utf8, false),
            Field::new(PAYLOAD_COLUMN, DataType::Utf8, false),
            Field::new(
                CREATED_AT_COLUMN,
                DataType::Timestamp(TimeUnit::Millisecond, None),
                false,
            ),
        ]))
    }))
}

/// Build the DuckLake table DDL for the Wendao event-lake table.
///
/// # Errors
///
/// Returns an error when the DuckLake catalog alias is not a valid DuckDB
/// identifier.
pub fn build_wendao_event_lake_table_sql(catalog_alias: &str) -> Result<String, String> {
    ensure_duckdb_identifier(catalog_alias, "DuckLake catalog")?;
    let catalog = quoted_duckdb_identifier(catalog_alias);
    let table = quoted_duckdb_identifier(WENDAO_EVENT_LAKE_EVENTS_TABLE);
    Ok(format!(
        "CREATE TABLE IF NOT EXISTS {catalog}.{table} (\
tenant_id VARCHAR, \
case_id VARCHAR, \
event_type VARCHAR, \
payload VARCHAR, \
created_at TIMESTAMP\
);"
    ))
}

/// Convert Wendao event records into one Arrow `RecordBatch`.
///
/// # Errors
///
/// Returns an error when Arrow rejects the assembled columns.
pub fn wendao_event_record_batch(events: &[WendaoEventRecord]) -> Result<RecordBatch, String> {
    let tenant_ids = events
        .iter()
        .map(|event| event.tenant_id.as_str())
        .collect::<Vec<_>>();
    let case_ids = events
        .iter()
        .map(|event| event.case_id.as_str())
        .collect::<Vec<_>>();
    let event_types = events
        .iter()
        .map(|event| event.event_type.as_str())
        .collect::<Vec<_>>();
    let payloads = events
        .iter()
        .map(|event| event.payload.to_string())
        .collect::<Vec<_>>();
    let created_at = events
        .iter()
        .map(|event| event.created_at.timestamp_millis())
        .collect::<Vec<_>>();

    RecordBatch::try_new(
        wendao_event_schema(),
        vec![
            Arc::new(StringArray::from(tenant_ids)) as ArrayRef,
            Arc::new(StringArray::from(case_ids)),
            Arc::new(StringArray::from(event_types)),
            Arc::new(StringArray::from(payloads)),
            Arc::new(TimestampMillisecondArray::from(created_at)),
        ],
    )
    .map_err(|error| format!("failed to build Wendao event Arrow batch: {error}"))
}

/// Validate that a record batch matches the Wendao event-lake schema.
///
/// # Errors
///
/// Returns an error when the incoming batch schema differs from the event-lake
/// append contract.
pub fn validate_wendao_event_batch(batch: &RecordBatch) -> Result<(), String> {
    let expected = wendao_event_schema();
    if batch.schema().as_ref() != expected.as_ref() {
        return Err(format!(
            "Wendao event batch schema mismatch: expected {:?}, got {:?}",
            expected,
            batch.schema()
        ));
    }
    Ok(())
}
