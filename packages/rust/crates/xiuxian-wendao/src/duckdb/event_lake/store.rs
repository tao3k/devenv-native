//! Append and query helpers for Wendao event rows in an attached `DuckLake`.

use arrow::record_batch::RecordBatch;
use xiuxian_db_store::duckdb::{
    DuckLakeRecordBatchAppender, DuckLakeTableRef, ensure_duckdb_identifier,
    quoted_duckdb_identifier,
};

use super::record::{WendaoEventRecord, WendaoEventTypeCount};
use super::schema::{
    WENDAO_EVENT_LAKE_EVENTS_TABLE, build_wendao_event_lake_table_sql, validate_wendao_event_batch,
    wendao_event_record_batch,
};

/// Default attached `DuckLake` catalog alias for Wendao event-lake experiments.
pub const WENDAO_EVENT_LAKE_DEFAULT_ALIAS: &str = "wendao_lake";

/// Default number of Wendao events converted into one Arrow batch.
pub const WENDAO_EVENT_APPEND_DEFAULT_BATCH_ROWS: usize = 4_096;

/// Reusable `Arrow` appender for the Wendao event table in an attached `DuckLake`.
pub struct WendaoEventLakeAppender<'conn> {
    inner: DuckLakeRecordBatchAppender<'conn>,
}

impl<'conn> WendaoEventLakeAppender<'conn> {
    /// Open a reusable appender for the Wendao event table.
    ///
    /// # Errors
    ///
    /// Returns an error when the catalog alias is invalid or `DuckDB` cannot
    /// open the target table appender.
    pub fn open(
        connection: &'conn duckdb::Connection,
        catalog_alias: &str,
    ) -> Result<Self, String> {
        ensure_duckdb_identifier(catalog_alias, "DuckLake catalog")?;
        let table = DuckLakeTableRef::main_schema(catalog_alias, WENDAO_EVENT_LAKE_EVENTS_TABLE);
        Ok(Self {
            inner: DuckLakeRecordBatchAppender::open(connection, &table)?,
        })
    }

    /// Append validated `Arrow` batches without flushing.
    ///
    /// # Errors
    ///
    /// Returns an error when a batch schema is invalid or `DuckDB` rejects the
    /// batch append.
    pub fn append_batches<I>(&mut self, batches: I) -> Result<usize, String>
    where
        I: IntoIterator<Item = RecordBatch>,
    {
        batches.into_iter().try_fold(0usize, |row_count, batch| {
            validate_wendao_event_batch(&batch)?;
            self.inner.append_batch(batch).map(|rows| row_count + rows)
        })
    }

    /// Convert Wendao event records into `Arrow` and append without flushing.
    ///
    /// # Errors
    ///
    /// Returns an error when `Arrow` batch creation fails or `DuckDB` rejects the
    /// batch append.
    pub fn append_events(&mut self, events: &[WendaoEventRecord]) -> Result<usize, String> {
        self.append_events_chunked(events, WENDAO_EVENT_APPEND_DEFAULT_BATCH_ROWS)
    }

    /// Convert Wendao event records into bounded `Arrow` batches and append
    /// without flushing.
    ///
    /// # Errors
    ///
    /// Returns an error when `rows_per_batch` is zero, `Arrow` batch creation
    /// fails, or `DuckDB` rejects a batch append.
    pub fn append_events_chunked(
        &mut self,
        events: &[WendaoEventRecord],
        rows_per_batch: usize,
    ) -> Result<usize, String> {
        if rows_per_batch == 0 {
            return Err("Wendao event append rows_per_batch must be greater than zero".to_string());
        }

        events
            .chunks(rows_per_batch)
            .try_fold(0usize, |row_count, chunk| {
                let batch = wendao_event_record_batch(chunk)?;
                self.append_batches(std::iter::once(batch))
                    .map(|rows| row_count + rows)
            })
    }

    /// Flush appended rows into `DuckDB`.
    ///
    /// # Errors
    ///
    /// Returns an error when `DuckDB` cannot flush the appender.
    pub fn flush(&mut self) -> Result<(), String> {
        self.inner.flush()
    }

    /// Return the number of rows appended through this appender.
    #[must_use]
    pub fn rows_appended(&self) -> usize {
        self.inner.rows_appended()
    }
}

/// Ensure the Wendao event table exists in one attached `DuckLake` catalog.
///
/// # Errors
///
/// Returns an error when DDL rendering fails or `DuckDB` rejects the statement.
pub fn ensure_wendao_event_lake_table(
    connection: &duckdb::Connection,
    catalog_alias: &str,
) -> Result<(), String> {
    let sql = build_wendao_event_lake_table_sql(catalog_alias)?;
    connection.execute_batch(sql.as_str()).map_err(|error| {
        format!("failed to create Wendao event-lake table in `{catalog_alias}`: {error}")
    })
}

/// Append `Arrow` batches into the Wendao event table in an attached `DuckLake`
/// catalog.
///
/// # Errors
///
/// Returns an error when a batch schema is invalid or when the db-store
/// `DuckLake` appender cannot write or flush the rows.
pub fn append_wendao_event_batches<I>(
    connection: &duckdb::Connection,
    catalog_alias: &str,
    batches: I,
) -> Result<usize, String>
where
    I: IntoIterator<Item = RecordBatch>,
{
    let mut appender = WendaoEventLakeAppender::open(connection, catalog_alias)?;
    let row_count = appender.append_batches(batches)?;
    appender.flush()?;
    Ok(row_count)
}

/// Convert Wendao event records into `Arrow` and append them into `DuckLake`.
///
/// # Errors
///
/// Returns an error when `Arrow` batch creation fails or when the append fails.
pub fn append_wendao_events(
    connection: &duckdb::Connection,
    catalog_alias: &str,
    events: &[WendaoEventRecord],
) -> Result<usize, String> {
    let mut appender = WendaoEventLakeAppender::open(connection, catalog_alias)?;
    let row_count = appender.append_events(events)?;
    appender.flush()?;
    Ok(row_count)
}

/// Query event counts grouped by `event_type`.
///
/// # Errors
///
/// Returns an error when the alias is invalid or `DuckDB` cannot execute the
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
