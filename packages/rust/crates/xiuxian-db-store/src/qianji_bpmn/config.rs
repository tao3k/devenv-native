use std::path::{Path, PathBuf};

use crate::duckdb::{DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig};

/// Default thread budget for local BPMN `DuckDB` data-store work.
pub const DEFAULT_QIANJI_BPMN_DUCKDB_THREADS: u64 = 2;

/// Resolved config for the Qianji BPMN `DuckDB` workflow data store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnDuckDbDataStoreConfig {
    runtime: DuckDbRuntimeConfig,
}

impl QianjiBpmnDuckDbDataStoreConfig {
    /// Builds a file-backed workflow data-store config with a sibling temp dir.
    #[must_use]
    pub fn file(database_path: impl Into<PathBuf>) -> Self {
        let database_path = database_path.into();
        let temp_directory = default_temp_directory(&database_path);
        Self::file_with_temp_directory(database_path, temp_directory)
    }

    /// Builds a file-backed workflow data-store config with an explicit temp dir.
    #[must_use]
    pub fn file_with_temp_directory(
        database_path: impl Into<PathBuf>,
        temp_directory: impl Into<PathBuf>,
    ) -> Self {
        Self::from_parts(
            DuckDbDatabasePath::File(database_path.into()),
            temp_directory,
        )
    }

    /// Builds an in-memory workflow data-store config with an explicit temp dir.
    #[must_use]
    pub fn in_memory(temp_directory: impl Into<PathBuf>) -> Self {
        Self::from_parts(DuckDbDatabasePath::InMemory, temp_directory)
    }

    /// Wraps one resolved generic `DuckDB` runtime config.
    #[must_use]
    pub fn from_runtime(runtime: DuckDbRuntimeConfig) -> Self {
        Self { runtime }
    }

    /// Accesses the resolved generic `DuckDB` runtime config.
    #[must_use]
    pub fn runtime(&self) -> &DuckDbRuntimeConfig {
        &self.runtime
    }

    /// Consumes this config into the generic `DuckDB` runtime config.
    #[must_use]
    pub fn into_runtime(self) -> DuckDbRuntimeConfig {
        self.runtime
    }

    fn from_parts(database_path: DuckDbDatabasePath, temp_directory: impl Into<PathBuf>) -> Self {
        Self {
            runtime: DuckDbRuntimeConfig {
                enabled: true,
                database_path,
                temp_directory: temp_directory.into(),
                threads: DEFAULT_QIANJI_BPMN_DUCKDB_THREADS,
                execution: DuckDbExecutionConfig {
                    preserve_insertion_order: true,
                    parquet_metadata_cache: false,
                    prefer_virtual_arrow: false,
                },
                memory_limit: None,
                max_temp_directory_size: None,
                materialize_threshold_rows: 0,
            },
        }
    }
}

fn default_temp_directory(database_path: &Path) -> PathBuf {
    database_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join(".qianji-duckdb-tmp")
}
