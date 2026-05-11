//! Feature-gated `DuckDB` connection opening and runtime setting application.

use std::fs;
use std::path::Path;

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
    ensure_duckdb_runtime_enabled(runtime)?;
    ensure_duckdb_database_parent(&runtime.database_path)?;
    let directories = ensure_duckdb_runtime_directories(runtime)?;
    let connection = open_duckdb_database(&runtime.database_path)?;
    apply_duckdb_runtime_settings(&connection, runtime, &directories)?;
    Ok(connection)
}

struct DuckDbRuntimeDirectories {
    home_directory: std::path::PathBuf,
    extension_directory: std::path::PathBuf,
}

fn ensure_duckdb_runtime_enabled(runtime: &DuckDbRuntimeConfig) -> Result<(), String> {
    runtime
        .enabled
        .then_some(())
        .ok_or_else(|| "DuckDB runtime is disabled".to_string())
}

fn ensure_duckdb_database_parent(database_path: &DuckDbDatabasePath) -> Result<(), String> {
    let DuckDbDatabasePath::File(path) = database_path else {
        return Ok(());
    };
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    create_named_directory(parent, "DuckDB database")
}

fn ensure_duckdb_runtime_directories(
    runtime: &DuckDbRuntimeConfig,
) -> Result<DuckDbRuntimeDirectories, String> {
    create_named_directory(&runtime.temp_directory, "DuckDB temp")?;
    let home_directory = runtime.temp_directory.join("home");
    create_named_directory(&home_directory, "DuckDB home")?;
    let extension_directory = runtime.temp_directory.join("extensions");
    create_named_directory(&extension_directory, "DuckDB extension")?;
    Ok(DuckDbRuntimeDirectories {
        home_directory,
        extension_directory,
    })
}

fn create_named_directory(path: &Path, label: &str) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| {
        format!(
            "failed to create {label} directory `{}`: {error}",
            path.display()
        )
    })
}

fn open_duckdb_database(
    database_path: &DuckDbDatabasePath,
) -> Result<::duckdb::Connection, String> {
    match database_path {
        DuckDbDatabasePath::InMemory => ::duckdb::Connection::open_in_memory()
            .map_err(|error| format!("failed to open in-memory DuckDB connection: {error}")),
        DuckDbDatabasePath::File(path) => ::duckdb::Connection::open(path).map_err(|error| {
            format!(
                "failed to open DuckDB database `{}`: {error}",
                path.display()
            )
        }),
    }
}

fn apply_duckdb_runtime_settings(
    connection: &::duckdb::Connection,
    runtime: &DuckDbRuntimeConfig,
    directories: &DuckDbRuntimeDirectories,
) -> Result<(), String> {
    let settings = duckdb_runtime_settings(runtime, directories);
    connection
        .execute_batch(format!("{};", settings.join(";\n")).as_str())
        .map_err(|error| format!("failed to initialize DuckDB settings: {error}"))
}

fn duckdb_runtime_settings(
    runtime: &DuckDbRuntimeConfig,
    directories: &DuckDbRuntimeDirectories,
) -> Vec<String> {
    let mut settings = base_duckdb_runtime_settings(runtime, directories);
    if runtime.execution.parquet_metadata_cache {
        settings.push("SET parquet_metadata_cache = true".to_string());
    }
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
    settings
}

fn base_duckdb_runtime_settings(
    runtime: &DuckDbRuntimeConfig,
    directories: &DuckDbRuntimeDirectories,
) -> Vec<String> {
    vec![
        format!(
            "SET home_directory = '{}'",
            escape_duckdb_setting_literal(directories.home_directory.to_string_lossy().as_ref())
        ),
        format!(
            "SET extension_directory = '{}'",
            escape_duckdb_setting_literal(
                directories.extension_directory.to_string_lossy().as_ref()
            )
        ),
        format!(
            "SET temp_directory = '{}'",
            escape_duckdb_setting_literal(runtime.temp_directory.to_string_lossy().as_ref())
        ),
        format!("SET threads = {}", runtime.threads),
        format!(
            "SET preserve_insertion_order = {}",
            runtime.execution.preserve_insertion_order
        ),
        "SET enable_profiling = 'no_output'".to_string(),
        "SET profiling_mode = 'standard'".to_string(),
    ]
}
