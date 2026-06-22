//! Wendao event-lake table and `Arrow` schema contracts.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use arrow::array::{ArrayRef, StringBuilder, TimestampMillisecondBuilder};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    duckdb::{ensure_duckdb_identifier, quoted_duckdb_identifier},
    validate_record_batch_schema_with_options,
};

use super::record::WendaoEventRecord;

/// Wendao-owned event table name inside an attached `DuckLake` catalog.
pub const WENDAO_EVENT_LAKE_EVENTS_TABLE: &str = "events";
const WENDAO_EVENT_LAKE_EVENTS_ARROW_TABLE: &str = "wendao_event_lake_events";

pub(crate) const TENANT_ID_COLUMN: &str = "tenant_id";
pub(crate) const CASE_ID_COLUMN: &str = "case_id";
pub(crate) const EVENT_TYPE_COLUMN: &str = "event_type";
pub(crate) const PAYLOAD_COLUMN: &str = "payload";
pub(crate) const CREATED_AT_COLUMN: &str = "created_at";

const DEFAULT_STRING_VALUE_BYTES: usize = 32;

/// Return the Arrow schema used for Wendao event-lake appends.
#[must_use]
pub fn wendao_event_schema() -> SchemaRef {
    static SCHEMA: OnceLock<SchemaRef> = OnceLock::new();
    Arc::clone(SCHEMA.get_or_init(|| schema_ref(&wendao_event_contract())))
}

/// Build the `DuckLake` table DDL for the Wendao event-lake table.
///
/// # Errors
///
/// Returns an error when the `DuckLake` catalog alias is not a valid `DuckDB`
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
    let row_count = events.len();
    let capacities = WendaoEventStringCapacities::from_events(events);
    let mut tenant_ids = string_builder_for(row_count, capacities.tenant_ids);
    let mut case_ids = string_builder_for(row_count, capacities.case_ids);
    let mut event_types = string_builder_for(row_count, capacities.event_types);
    let mut payloads = string_builder_for(row_count, capacities.payloads);
    let mut created_at = TimestampMillisecondBuilder::with_capacity(row_count);

    for event in events {
        tenant_ids.append_value(event.tenant_id.as_str());
        case_ids.append_value(event.case_id.as_str());
        event_types.append_value(event.event_type.as_str());
        payloads.append_value(event.payload_json());
        created_at.append_value(event.created_at.timestamp_millis());
    }

    let batch = RecordBatch::try_new(
        wendao_event_schema(),
        vec![
            Arc::new(tenant_ids.finish()) as ArrayRef,
            Arc::new(case_ids.finish()),
            Arc::new(event_types.finish()),
            Arc::new(payloads.finish()),
            Arc::new(created_at.finish()),
        ],
    )
    .map_err(|error| format!("failed to build Wendao event Arrow batch: {error}"))?;
    validate_wendao_event_batch(&batch)?;
    Ok(batch)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct WendaoEventStringCapacities {
    tenant_ids: usize,
    case_ids: usize,
    event_types: usize,
    payloads: usize,
}

impl WendaoEventStringCapacities {
    fn from_events(events: &[WendaoEventRecord]) -> Self {
        let mut capacities = Self::default();
        for event in events {
            capacities.tenant_ids += event.tenant_id.len();
            capacities.case_ids += event.case_id.len();
            capacities.event_types += event.event_type.len();
            capacities.payloads += event.payload_json().len();
        }
        capacities
    }
}

fn string_builder_for(row_count: usize, exact_capacity: usize) -> StringBuilder {
    StringBuilder::with_capacity(
        row_count,
        exact_capacity.max(row_count * DEFAULT_STRING_VALUE_BYTES),
    )
}

/// Validate that a record batch matches the Wendao event-lake schema.
///
/// # Errors
///
/// Returns an error when the incoming batch schema differs from the event-lake
/// append contract.
pub fn validate_wendao_event_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_record_batch_schema_with_options(
        batch,
        &wendao_event_contract(),
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| format!("Wendao event batch schema validation failed: {error}"))
}

fn wendao_event_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        WENDAO_EVENT_LAKE_EVENTS_ARROW_TABLE,
        true,
        vec![
            utf8_column(TENANT_ID_COLUMN),
            utf8_column(CASE_ID_COLUMN),
            utf8_column(EVENT_TYPE_COLUMN),
            utf8_column(PAYLOAD_COLUMN),
            ArrowSchemaColumn::new(CREATED_AT_COLUMN, ArrowSchemaDataType::TimestampMillisecond),
        ],
    )
}

fn schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
    Arc::new(build_arrow_schema(
        contract,
        [(
            WENDAO_TABLE_METADATA_KEY.to_string(),
            contract.table_name().to_string(),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    ))
}

const fn utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}
