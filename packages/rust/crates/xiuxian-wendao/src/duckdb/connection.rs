use std::fs;

use duckdb::Connection;

use super::runtime::resolve_enabled_search_duckdb_runtime;
use crate::duckdb::{DuckDbDatabasePath, SearchDuckDbRuntimeConfig};

fn escape_duckdb_setting_literal(value: &str) -> String {
    value.replace('\'', "''")
}

/// Feature-gated host-owned `DuckDB` connection wrapper for bounded analytics.
pub struct SearchDuckDbConnection {
    connection: Connection,
    runtime: SearchDuckDbRuntimeConfig,
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
        let connection = open_search_duckdb_connection(&runtime)?;
        Ok(Self {
            connection,
            runtime,
        })
    }

    /// Access the underlying `DuckDB` connection.
    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Access the runtime config used to open this connection.
    #[must_use]
    pub fn runtime(&self) -> &SearchDuckDbRuntimeConfig {
        &self.runtime
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
) -> Result<Connection, String> {
    if !runtime.enabled {
        return Err("search DuckDB runtime is disabled".to_string());
    }

    match &runtime.database_path {
        DuckDbDatabasePath::InMemory => {}
        DuckDbDatabasePath::File(path) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to create search DuckDB database directory `{}`: {error}",
                        parent.display()
                    )
                })?;
            }
        }
    }

    fs::create_dir_all(&runtime.temp_directory).map_err(|error| {
        format!(
            "failed to create search DuckDB temp directory `{}`: {error}",
            runtime.temp_directory.display()
        )
    })?;

    let connection = match &runtime.database_path {
        DuckDbDatabasePath::InMemory => Connection::open_in_memory().map_err(|error| {
            format!("failed to open in-memory search DuckDB connection: {error}")
        })?,
        DuckDbDatabasePath::File(path) => Connection::open(path).map_err(|error| {
            format!(
                "failed to open search DuckDB database `{}`: {error}",
                path.display()
            )
        })?,
    };

    let escaped_temp_directory =
        escape_duckdb_setting_literal(runtime.temp_directory.to_string_lossy().as_ref());
    let mut settings = vec![
        format!("SET temp_directory = '{escaped_temp_directory}'"),
        format!("SET threads = {}", runtime.threads),
        format!(
            "SET preserve_insertion_order = {}",
            runtime.preserve_insertion_order
        ),
        format!(
            "SET parquet_metadata_cache = {}",
            runtime.parquet_metadata_cache
        ),
        "SET enable_profiling = 'no_output'".to_string(),
        "SET profiling_mode = 'standard'".to_string(),
    ];
    if let Some(memory_limit) = runtime.memory_limit.as_deref() {
        settings.push(format!(
            "SET memory_limit = '{}'",
            escape_duckdb_setting_literal(memory_limit)
        ));
    }
    if let Some(max_temp_directory_size) = runtime.max_temp_directory_size.as_deref() {
        settings.push(format!(
            "SET max_temp_directory_size = '{}'",
            escape_duckdb_setting_literal(max_temp_directory_size)
        ));
    }
    connection
        .execute_batch(format!("{};", settings.join(";\n")).as_str())
        .map_err(|error| format!("failed to initialize search DuckDB settings: {error}"))?;

    Ok(connection)
}
