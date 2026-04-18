use std::fs;
use std::sync::Arc;

#[cfg(feature = "duckdb")]
use std::path::Path;

#[cfg(feature = "duckdb")]
use arrow::array::Int64Array;
use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use serial_test::serial;

use crate::duckdb::{
    DataFusionLocalRelationEngine, DuckDbDatabasePath, LocalRelationEngine,
    LocalRelationEngineKind, resolve_search_duckdb_runtime,
};
#[cfg(feature = "duckdb")]
use crate::duckdb::{
    DuckDbLocalRelationEngine, DuckDbRegistrationStrategy, LocalRelationRegistrationHint,
    ParquetQueryEngine, SearchDuckDbExecutionConfig, SearchDuckDbRuntimeConfig,
};
use crate::link_graph::set_link_graph_wendao_config_override;
#[cfg(feature = "duckdb")]
use xiuxian_wendao_runtime::config::{
    DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS, DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE,
    DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW, DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER,
    DEFAULT_SEARCH_DUCKDB_THREADS, resolve_search_duckdb_runtime_with_settings,
};

mod configured;
mod relation_engine;
mod runtime;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn write_search_duckdb_runtime_override(
    body: &str,
) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(&config_path, body)?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());
    Ok(temp)
}

#[cfg(feature = "duckdb")]
fn load_toml_settings_from_path(
    path: &Path,
) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
    let parsed: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let encoded = serde_json::to_string(&parsed)?;
    Ok(serde_json::from_str(&encoded)?)
}

#[cfg(feature = "duckdb")]
fn in_memory_search_duckdb_runtime(root: &Path) -> SearchDuckDbRuntimeConfig {
    SearchDuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::InMemory,
        temp_directory: root.join(".cache/duckdb-test/tmp"),
        threads: 2,
        execution: SearchDuckDbExecutionConfig {
            preserve_insertion_order: DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER,
            parquet_metadata_cache: DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE,
            prefer_virtual_arrow: DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW,
        },
        memory_limit: None,
        max_temp_directory_size: None,
        materialize_threshold_rows: DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS,
    }
}
