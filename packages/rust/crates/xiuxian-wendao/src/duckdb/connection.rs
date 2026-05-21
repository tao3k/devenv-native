//! `duckdb::connection` owns Wendao duckdb connection behavior.

use super::runtime::resolve_enabled_search_duckdb_runtime;
use crate::duckdb::SearchDuckDbRuntimeConfig;
use xiuxian_db_store::duckdb::{
    DuckDbConnection as GenericDuckDbConnection, open_duckdb_connection as open_generic_connection,
};

/// Feature-gated host-owned `DuckDB` connection wrapper for bounded analytics.
pub struct SearchDuckDbConnection {
    inner: GenericDuckDbConnection,
}

impl SearchDuckDbConnection {
    /// Open a configured bounded `DuckDB` connection from merged Wendao settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the runtime is disabled or when the connection
    /// cannot be opened and initialized.
    pub fn configured() -> Result<Self, String> {
        let runtime = resolve_enabled_search_duckdb_runtime("configured search DuckDB connection")?;
        Self::from_runtime(runtime)
    }

    /// Open a bounded `DuckDB` connection from one resolved runtime config.
    ///
    /// # Errors
    ///
    /// Returns an error when the connection cannot be opened and initialized.
    pub fn from_runtime(runtime: SearchDuckDbRuntimeConfig) -> Result<Self, String> {
        GenericDuckDbConnection::from_runtime(runtime).map(|inner| Self { inner })
    }

    /// Access the underlying `DuckDB` connection.
    #[must_use]
    pub fn connection(&self) -> &duckdb::Connection {
        self.inner.connection()
    }

    /// Access the runtime config used to open this connection.
    #[must_use]
    pub fn runtime(&self) -> &SearchDuckDbRuntimeConfig {
        self.inner.runtime()
    }
}

/// Open one bounded `DuckDB` connection from a resolved runtime config.
///
/// # Errors
///
/// Returns an error when the runtime is disabled, when required directories
/// cannot be created, or when `DuckDB` rejects the initialization settings.
pub fn open_search_duckdb_connection(
    runtime: &SearchDuckDbRuntimeConfig,
) -> Result<duckdb::Connection, String> {
    open_generic_connection(runtime).map_err(|error| format!("search {error}"))
}
