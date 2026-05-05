//! Typed Wendao event-lake handle over an attached `DuckLake` catalog.

use arrow::record_batch::RecordBatch;
use xiuxian_db_store::duckdb::{
    DuckLakeAttachConfig, DuckLakeTableRef, attach_ducklake, ensure_duckdb_identifier,
};

use super::WENDAO_EVENT_LAKE_EVENTS_TABLE;
use super::query::{WendaoEventQuery, query_wendao_events};
use super::record::{WendaoEventRecord, WendaoEventTypeCount};
use super::store::{
    WENDAO_EVENT_LAKE_DEFAULT_ALIAS, WendaoEventLakeAppender, append_wendao_event_batches,
    append_wendao_events, ensure_wendao_event_lake_table, query_wendao_event_type_counts,
};

/// Wendao-owned handle for one attached `DuckLake` event-lake catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoEventLake {
    catalog_alias: String,
}

impl WendaoEventLake {
    /// Build a handle for an already attached `DuckLake` catalog.
    ///
    /// # Errors
    ///
    /// Returns an error when the catalog alias is not a valid `DuckDB`
    /// identifier.
    pub fn attached(catalog_alias: impl Into<String>) -> Result<Self, String> {
        let catalog_alias = catalog_alias.into();
        ensure_duckdb_identifier(&catalog_alias, "DuckLake catalog")?;
        Ok(Self { catalog_alias })
    }

    /// Build a handle for the default Wendao event-lake catalog alias.
    #[must_use]
    pub fn default_alias() -> Self {
        Self {
            catalog_alias: WENDAO_EVENT_LAKE_DEFAULT_ALIAS.to_string(),
        }
    }

    /// Attach the configured `DuckLake` catalog and ensure the event table.
    ///
    /// # Errors
    ///
    /// Returns an error when the `DuckLake` attach operation fails, the alias is
    /// invalid, or the event table cannot be created.
    pub fn attach(
        connection: &duckdb::Connection,
        config: &DuckLakeAttachConfig,
    ) -> Result<Self, String> {
        attach_ducklake(connection, config)?;
        let lake = Self::attached(config.alias.as_str())?;
        lake.ensure_table(connection)?;
        Ok(lake)
    }

    /// Access the attached `DuckLake` catalog alias.
    #[must_use]
    pub fn catalog_alias(&self) -> &str {
        self.catalog_alias.as_str()
    }

    /// Build the fully qualified event table reference for this lake.
    #[must_use]
    pub fn events_table_ref(&self) -> DuckLakeTableRef {
        DuckLakeTableRef::main_schema(self.catalog_alias.as_str(), WENDAO_EVENT_LAKE_EVENTS_TABLE)
    }

    /// Ensure the Wendao event table exists.
    ///
    /// # Errors
    ///
    /// Returns an error when DDL rendering fails or `DuckDB` rejects the
    /// statement.
    pub fn ensure_table(&self, connection: &duckdb::Connection) -> Result<(), String> {
        ensure_wendao_event_lake_table(connection, self.catalog_alias.as_str())
    }

    /// Append Arrow batches into the Wendao event table.
    ///
    /// # Errors
    ///
    /// Returns an error when a batch schema is invalid or when `DuckDB` rejects
    /// the appender operation.
    pub fn append_batches<I>(
        &self,
        connection: &duckdb::Connection,
        batches: I,
    ) -> Result<usize, String>
    where
        I: IntoIterator<Item = RecordBatch>,
    {
        append_wendao_event_batches(connection, self.catalog_alias.as_str(), batches)
    }

    /// Open a reusable event-lake appender for high-throughput ingestion.
    ///
    /// # Errors
    ///
    /// Returns an error when `DuckDB` cannot open the event-table appender.
    pub fn open_appender<'conn>(
        &self,
        connection: &'conn duckdb::Connection,
    ) -> Result<WendaoEventLakeAppender<'conn>, String> {
        WendaoEventLakeAppender::open(connection, self.catalog_alias.as_str())
    }

    /// Convert and append Wendao event records.
    ///
    /// # Errors
    ///
    /// Returns an error when Arrow batch creation fails or when the append
    /// fails.
    pub fn append_events(
        &self,
        connection: &duckdb::Connection,
        events: &[WendaoEventRecord],
    ) -> Result<usize, String> {
        append_wendao_events(connection, self.catalog_alias.as_str(), events)
    }

    /// Query event counts grouped by `event_type`.
    ///
    /// # Errors
    ///
    /// Returns an error when `DuckDB` cannot execute or read the query.
    pub fn event_type_counts(
        &self,
        connection: &duckdb::Connection,
    ) -> Result<Vec<WendaoEventTypeCount>, String> {
        query_wendao_event_type_counts(connection, self.catalog_alias.as_str())
    }

    /// Query bounded event rows from this lake.
    ///
    /// # Errors
    ///
    /// Returns an error when the query is invalid or `DuckDB` cannot execute or
    /// read the row query.
    pub fn query_events(
        &self,
        connection: &duckdb::Connection,
        query: &WendaoEventQuery,
    ) -> Result<Vec<WendaoEventRecord>, String> {
        query_wendao_events(connection, self.catalog_alias.as_str(), query)
    }
}
