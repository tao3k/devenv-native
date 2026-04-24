use std::fs;

use super::{DuckDbDatabasePath, DuckDbRuntimeConfig};

fn escape_duckdb_setting_literal(value: &str) -> String {
    value.replace('\'', "''")
}

/// Feature-gated `DuckDB` connection wrapper for bounded local storage.
pub struct DuckDbConnection {
    connection: ::duckdb::Connection,
    runtime: DuckDbRuntimeConfig,
}

impl DuckDbConnection {
    /// Open a bounded `DuckDB` connection from one resolved runtime config.
    ///
    /// # Errors
    ///
    /// Returns an error when the connection cannot be opened and initialized.
    pub fn from_runtime(runtime: DuckDbRuntimeConfig) -> Result<Self, String> {
        let connection = open_duckdb_connection(&runtime)?;
        Ok(Self {
            connection,
            runtime,
        })
    }

    /// Access the underlying `DuckDB` connection.
    #[must_use]
    pub fn connection(&self) -> &::duckdb::Connection {
        &self.connection
    }

    /// Access the runtime config used to open this connection.
    #[must_use]
    pub fn runtime(&self) -> &DuckDbRuntimeConfig {
        &self.runtime
    }
}

/// Open one bounded `DuckDB` connection from a resolved runtime config.
///
/// # Errors
///
/// Returns an error when the runtime is disabled, when required directories
/// cannot be created, or when `DuckDB` rejects the initialization settings.
pub fn open_duckdb_connection(
    runtime: &DuckDbRuntimeConfig,
) -> Result<::duckdb::Connection, String> {
    if !runtime.enabled {
        return Err("DuckDB runtime is disabled".to_string());
    }

    match &runtime.database_path {
        DuckDbDatabasePath::InMemory => {}
        DuckDbDatabasePath::File(path) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to create DuckDB database directory `{}`: {error}",
                        parent.display()
                    )
                })?;
            }
        }
    }

    fs::create_dir_all(&runtime.temp_directory).map_err(|error| {
        format!(
            "failed to create DuckDB temp directory `{}`: {error}",
            runtime.temp_directory.display()
        )
    })?;

    let connection = match &runtime.database_path {
        DuckDbDatabasePath::InMemory => ::duckdb::Connection::open_in_memory()
            .map_err(|error| format!("failed to open in-memory DuckDB connection: {error}"))?,
        DuckDbDatabasePath::File(path) => ::duckdb::Connection::open(path).map_err(|error| {
            format!(
                "failed to open DuckDB database `{}`: {error}",
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
            runtime.execution.preserve_insertion_order
        ),
        format!(
            "SET parquet_metadata_cache = {}",
            runtime.execution.parquet_metadata_cache
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
        .map_err(|error| format!("failed to initialize DuckDB settings: {error}"))?;

    Ok(connection)
}
