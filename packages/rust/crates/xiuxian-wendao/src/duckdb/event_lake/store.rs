//! Append and query helpers for Wendao event rows in an attached `DuckLake`.

use arrow::record_batch::RecordBatch;
use xiuxian_db_store::duckdb::{
    DuckLakeTableRef, append_ducklake_record_batches, ensure_duckdb_identifier,
    quoted_duckdb_identifier,
};

use super::record::{WendaoEventRecord, WendaoEventTypeCount};
use super::schema::{
    WENDAO_EVENT_LAKE_EVENTS_TABLE, build_wendao_event_lake_table_sql, validate_wendao_event_batch,
    wendao_event_record_batch,
};

/// Default attached DuckLake catalog alias for Wendao event-lake experiments.
pub const WENDAO_EVENT_LAKE_DEFAULT_ALIAS: &str = "wendao_lake";

/// Ensure the Wendao event table exists in one attached DuckLake catalog.
///
/// # Errors
///
/// Returns an error when DDL rendering fails or DuckDB rejects the statement.
pub fn ensure_wendao_event_lake_table(
    connection: &duckdb::Connection,
    catalog_alias: &str,
) -> Result<(), String> {
    let sql = build_wendao_event_lake_table_sql(catalog_alias)?;
    connection.execute_batch(sql.as_str()).map_err(|error| {
        format!("failed to create Wendao event-lake table in `{catalog_alias}`: {error}")
    })
}

/// Append Arrow batches into the Wendao event table in an attached DuckLake
/// catalog.
///
/// # Errors
///
/// Returns an error when a batch schema is invalid or when the db-store
/// DuckLake appender cannot write or flush the rows.
pub fn append_wendao_event_batches(
    connection: &duckdb::Connection,
    catalog_alias: &str,
    batches: Vec<RecordBatch>,
) -> Result<usize, String> {
    ensure_duckdb_identifier(catalog_alias, "DuckLake catalog")?;
    for batch in &batches {
        validate_wendao_event_batch(batch)?;
    }
    append_ducklake_record_batches(
        connection,
        &DuckLakeTableRef::main_schema(catalog_alias, WENDAO_EVENT_LAKE_EVENTS_TABLE),
        batches,
    )
}

/// Convert Wendao event records into Arrow and append them into DuckLake.
///
/// # Errors
///
/// Returns an error when Arrow batch creation fails or when the append fails.
pub fn append_wendao_events(
    connection: &duckdb::Connection,
    catalog_alias: &str,
    events: &[WendaoEventRecord],
) -> Result<usize, String> {
    let batch = wendao_event_record_batch(events)?;
    append_wendao_event_batches(connection, catalog_alias, vec![batch])
}

/// Query event counts grouped by `event_type`.
///
/// # Errors
///
/// Returns an error when the alias is invalid or DuckDB cannot execute the
/// query.
pub fn query_wendao_event_type_counts(
    connection: &duckdb::Connection,
    catalog_alias: &str,
) -> Result<Vec<WendaoEventTypeCount>, String> {
    ensure_duckdb_identifier(catalog_alias, "DuckLake catalog")?;
    let catalog = quoted_duckdb_identifier(catalog_alias);
    let table = quoted_duckdb_identifier(WENDAO_EVENT_LAKE_EVENTS_TABLE);
    let sql = format!(
        "SELECT event_type, COUNT(*)::BIGINT AS event_count \
         FROM {catalog}.{table} \
         GROUP BY event_type \
         ORDER BY event_type"
    );
    let mut statement = connection
        .prepare(sql.as_str())
        .map_err(|error| format!("failed to prepare Wendao event-lake count query: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(WendaoEventTypeCount {
                event_type: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|error| format!("failed to execute Wendao event-lake count query: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read Wendao event-lake count row: {error}"))
}
