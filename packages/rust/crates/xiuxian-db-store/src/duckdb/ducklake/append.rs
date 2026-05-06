//! `Arrow` appender helpers for attached `DuckLake` tables.

use super::DuckLakeTableRef;

/// Reusable `Arrow` appender for one attached `DuckLake` table.
///
/// Keeping this appender open lets event producers amortize appender creation
/// and flush costs across multiple `RecordBatch` chunks.
pub struct DuckLakeRecordBatchAppender<'conn> {
    table: DuckLakeTableRef,
    appender: ::duckdb::Appender<'conn>,
    row_count: usize,
}

impl<'conn> DuckLakeRecordBatchAppender<'conn> {
    /// Open a reusable `Arrow` appender for one attached `DuckLake` table.
    ///
    /// # Errors
    ///
    /// Returns an error when the table reference is invalid or `DuckDB` cannot
    /// open an appender for the target table.
    pub fn open(
        connection: &'conn ::duckdb::Connection,
        table: &DuckLakeTableRef,
    ) -> Result<Self, String> {
        table.validate()?;
        let appender = connection
            .appender_to_catalog_and_db(&table.table_name, &table.catalog_alias, &table.schema_name)
            .map_err(|error| open_appender_error(table, &error))?;
        Ok(Self {
            table: table.clone(),
            appender,
            row_count: 0,
        })
    }

    /// Append one `Arrow` record batch without flushing.
    ///
    /// # Errors
    ///
    /// Returns an error when `DuckDB` rejects the batch schema or data chunk.
    pub fn append_batch(
        &mut self,
        batch: ::duckdb::arrow::record_batch::RecordBatch,
    ) -> Result<usize, String> {
        let rows = batch.num_rows();
        self.appender
            .append_record_batch(batch)
            .map_err(|error| append_batch_error(&self.table, &error))?;
        self.row_count += rows;
        Ok(rows)
    }

    /// Append `Arrow` record batches without flushing.
    ///
    /// # Errors
    ///
    /// Returns an error when `DuckDB` rejects any batch schema or data chunk.
    pub fn append_batches<I>(&mut self, batches: I) -> Result<usize, String>
    where
        I: IntoIterator<Item = ::duckdb::arrow::record_batch::RecordBatch>,
    {
        let mut appended_rows = 0;
        for batch in batches {
            appended_rows += self.append_batch(batch)?;
        }
        Ok(appended_rows)
    }

    /// Flush appended rows into `DuckDB`.
    ///
    /// # Errors
    ///
    /// Returns an error when `DuckDB` cannot flush the appender.
    pub fn flush(&mut self) -> Result<(), String> {
        self.appender
            .flush()
            .map_err(|error| flush_appender_error(&self.table, &error))
    }

    /// Return the number of rows appended through this appender.
    #[must_use]
    pub fn rows_appended(&self) -> usize {
        self.row_count
    }
}

/// Append `Arrow` record batches into one existing `DuckLake` table through
/// `DuckDB`'s `Arrow` appender.
///
/// # Errors
///
/// Returns an error when the table reference is invalid, the target table does
/// not exist, a batch schema cannot be appended, or the appender cannot flush.
pub fn append_ducklake_record_batches<I>(
    connection: &::duckdb::Connection,
    table: &DuckLakeTableRef,
    batches: I,
) -> Result<usize, String>
where
    I: IntoIterator<Item = ::duckdb::arrow::record_batch::RecordBatch>,
{
    let mut appender = DuckLakeRecordBatchAppender::open(connection, table)?;
    let row_count = appender.append_batches(batches)?;
    appender.flush()?;
    Ok(row_count)
}

fn open_appender_error(table: &DuckLakeTableRef, error: &::duckdb::Error) -> String {
    format!(
        "failed to open DuckLake appender for `{}`.`{}`.`{}`: {error}",
        table.catalog_alias, table.schema_name, table.table_name
    )
}

fn append_batch_error(table: &DuckLakeTableRef, error: &::duckdb::Error) -> String {
    format!(
        "failed to append Arrow batch into DuckLake table `{}`.`{}`.`{}`: {error}",
        table.catalog_alias, table.schema_name, table.table_name
    )
}

fn flush_appender_error(table: &DuckLakeTableRef, error: &::duckdb::Error) -> String {
    format!(
        "failed to flush DuckLake appender for `{}`.`{}`.`{}`: {error}",
        table.catalog_alias, table.schema_name, table.table_name
    )
}
