//! `DuckDB` runtime path and execution configuration types.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Resolved database location for bounded `DuckDB` storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuckDbDatabasePath {
    /// Use `DuckDB`'s own ephemeral in-process database.
    InMemory,
    /// Use one bounded on-disk `DuckDB` database file.
    File(PathBuf),
}

/// Runtime-owned `DuckDB` execution toggles for bounded storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuckDbExecutionConfig {
    /// Whether `DuckDB` may reorder results that do not contain explicit
    /// `ORDER BY` clauses.
    pub preserve_insertion_order: bool,
    /// Whether `DuckDB` should cache Parquet metadata across repeated scans of
    /// the same files.
    pub parquet_metadata_cache: bool,
    /// Prefer Arrow virtual-table registration when possible.
    pub prefer_virtual_arrow: bool,
}

/// Runtime-owned `DuckDB` config for bounded local storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuckDbRuntimeConfig {
    /// Enable the bounded `DuckDB` lane.
    pub enabled: bool,
    /// Resolved database location.
    pub database_path: DuckDbDatabasePath,
    /// Resolved temp/spill directory.
    pub temp_directory: PathBuf,
    /// Maximum threads `DuckDB` should use for bounded storage work.
    pub threads: u64,
    /// Execution toggles for bounded `DuckDB` storage.
    #[serde(flatten)]
    pub execution: DuckDbExecutionConfig,
    /// Optional explicit `DuckDB` buffer-manager memory limit, e.g. `4GB`.
    pub memory_limit: Option<String>,
    /// Optional explicit `DuckDB` spill limit for the configured temp directory,
    /// e.g. `20GB`.
    pub max_temp_directory_size: Option<String>,
    /// Row threshold for choosing bounded materialization over purely virtual registration.
    pub materialize_threshold_rows: u64,
}
