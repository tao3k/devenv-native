//! `Arrow` appender helpers for attached `DuckLake` tables.

use super::DuckLakeTableRef;

/// Append Arrow record batches into one existing DuckLake table through
/// DuckDB's Arrow appender.
///
/// # Errors
///
/// Returns an error when the table reference is invalid, the target table does
/// not exist, a batch schema cannot be appended, or the appender cannot flush.
pub fn append_ducklake_record_batches(
    connection: &::duckdb::Connection,
    table: &DuckLakeTableRef,
    batches: Vec<::duckdb::arrow::record_batch::RecordBatch>,
) -> Result<usize, String> {
    table.validate()?;
    let mut appender = connection
        .appender_to_catalog_and_db(&table.table_name, &table.catalog_alias, &table.schema_name)
        .map_err(|error| {
            format!(
                "failed to open DuckLake appender for `{}`.`{}`.`{}`: {error}",
                table.catalog_alias, table.schema_name, table.table_name
            )
        })?;
    let mut row_count = 0;
    for batch in batches {
        row_count += batch.num_rows();
        appender.append_record_batch(batch).map_err(|error| {
            format!(
                "failed to append Arrow batch into DuckLake table `{}`.`{}`.`{}`: {error}",
                table.catalog_alias, table.schema_name, table.table_name
            )
        })?;
    }
    appender.flush().map_err(|error| {
        format!(
            "failed to flush DuckLake appender for `{}`.`{}`.`{}`: {error}",
            table.catalog_alias, table.schema_name, table.table_name
        )
    })?;
    Ok(row_count)
}
