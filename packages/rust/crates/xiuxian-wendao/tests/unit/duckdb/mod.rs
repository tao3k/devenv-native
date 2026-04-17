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

#[test]
#[serial]
fn resolve_search_duckdb_runtime_reads_override_values() -> TestResult {
    let temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ".data/duckdb/search.db"
temp_directory = ".cache/duckdb/custom-tmp"
threads = 6
preserve_insertion_order = true
parquet_metadata_cache = false
memory_limit = "3GB"
max_temp_directory_size = "9GB"
materialize_threshold_rows = 123
prefer_virtual_arrow = false
"#,
    )?;

    let runtime = resolve_search_duckdb_runtime();
    assert!(runtime.enabled);
    assert_eq!(
        runtime.database_path,
        DuckDbDatabasePath::File(temp.path().join(".data/duckdb/search.db"))
    );
    assert_eq!(
        runtime.temp_directory,
        temp.path().join(".cache/duckdb/custom-tmp")
    );
    assert_eq!(runtime.threads, 6);
    assert!(runtime.execution.preserve_insertion_order);
    assert!(!runtime.execution.parquet_metadata_cache);
    assert_eq!(runtime.memory_limit.as_deref(), Some("3GB"));
    assert_eq!(runtime.max_temp_directory_size.as_deref(), Some("9GB"));
    assert_eq!(runtime.materialize_threshold_rows, 123);
    assert!(!runtime.execution.prefer_virtual_arrow);

    Ok(())
}

#[cfg(feature = "duckdb")]
#[test]
fn embedded_search_duckdb_defaults_follow_system_profile() -> TestResult {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let resource_path = crate_root.join("resources/config/wendao.toml");
    let settings = load_toml_settings_from_path(resource_path.as_path())?;
    let runtime = resolve_search_duckdb_runtime_with_settings(crate_root, &settings);

    assert!(runtime.enabled);
    assert_eq!(runtime.database_path, DuckDbDatabasePath::InMemory);
    assert_eq!(runtime.temp_directory, crate_root.join(".cache/duckdb/tmp"));
    assert_eq!(runtime.threads, DEFAULT_SEARCH_DUCKDB_THREADS);
    assert!(runtime.execution.preserve_insertion_order);
    assert!(!runtime.execution.parquet_metadata_cache);
    assert_eq!(runtime.memory_limit, None);
    assert_eq!(runtime.max_temp_directory_size, None);

    Ok(())
}

#[tokio::test]
async fn datafusion_local_relation_engine_registers_and_queries_batches() -> TestResult {
    let engine = DataFusionLocalRelationEngine::new_with_information_schema();
    assert_eq!(engine.kind(), LocalRelationEngineKind::DataFusion);

    let schema = Arc::new(Schema::new(vec![Field::new("name", DataType::Utf8, false)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["alpha", "beta"]))],
    )?;

    engine.register_record_batches("ducks", schema, vec![batch])?;
    let result = engine
        .query_batches("select name from ducks order by name")
        .await
        .map_err(std::io::Error::other)?;

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].num_rows(), 2);
    assert_eq!(result[0].schema().field(0).name(), "name");
    Ok(())
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn duckdb_local_relation_engine_registers_and_queries_batches() -> TestResult {
    let temp = tempfile::tempdir()?;
    let engine =
        DuckDbLocalRelationEngine::from_runtime(in_memory_search_duckdb_runtime(temp.path()))
            .map_err(std::io::Error::other)?;
    assert_eq!(engine.kind(), LocalRelationEngineKind::DuckDb);

    let schema = Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new("line_count", DataType::Int64, false),
    ]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from(vec!["beta", "alpha"])),
            Arc::new(Int64Array::from(vec![2_i64, 1_i64])),
        ],
    )?;

    engine.register_record_batches("ducks", schema, vec![batch])?;
    assert_eq!(
        engine.registered_strategy("ducks")?,
        Some(DuckDbRegistrationStrategy::VirtualArrow)
    );
    let result = engine
        .query_batches("select name, line_count from ducks order by line_count")
        .await
        .map_err(std::io::Error::other)?;

    assert!(engine.last_query_temp_storage_peak_bytes().is_some());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].num_rows(), 2);
    let names = result[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| std::io::Error::other("missing Utf8 name column"))?;
    let counts = result[0]
        .column(1)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| std::io::Error::other("missing Int64 line_count column"))?;
    assert_eq!(names.value(0), "alpha");
    assert_eq!(names.value(1), "beta");
    assert_eq!(counts.value(0), 1);
    assert_eq!(counts.value(1), 2);
    Ok(())
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn duckdb_local_relation_engine_materializes_when_threshold_is_reached() -> TestResult {
    let temp = tempfile::tempdir()?;
    let mut runtime = in_memory_search_duckdb_runtime(temp.path());
    runtime.materialize_threshold_rows = 2;
    let engine = DuckDbLocalRelationEngine::from_runtime(runtime).map_err(std::io::Error::other)?;

    let schema = Arc::new(Schema::new(vec![Field::new("name", DataType::Utf8, false)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["beta", "alpha"]))],
    )?;

    engine.register_record_batches("ducks", schema, vec![batch])?;
    assert_eq!(
        engine.registered_strategy("ducks")?,
        Some(DuckDbRegistrationStrategy::MaterializedAppender)
    );
    let result = engine
        .query_batches("select name from ducks order by name")
        .await
        .map_err(std::io::Error::other)?;

    assert!(engine.last_query_temp_storage_peak_bytes().is_some());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].num_rows(), 2);
    Ok(())
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn duckdb_local_relation_engine_materializes_when_repeated_use_is_hint() -> TestResult {
    let temp = tempfile::tempdir()?;
    let mut runtime = in_memory_search_duckdb_runtime(temp.path());
    runtime.materialize_threshold_rows = 100;
    let engine = DuckDbLocalRelationEngine::from_runtime(runtime).map_err(std::io::Error::other)?;

    let schema = Arc::new(Schema::new(vec![Field::new("name", DataType::Utf8, false)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["beta", "alpha"]))],
    )?;

    engine.register_record_batches_with_hint(
        "ducks",
        schema,
        vec![batch],
        LocalRelationRegistrationHint::RepeatedUse,
    )?;
    assert_eq!(
        engine.registered_strategy("ducks")?,
        Some(DuckDbRegistrationStrategy::MaterializedAppender)
    );
    assert_eq!(
        engine.relation_materialization_state("ducks"),
        Some(crate::duckdb::LocalRelationMaterializationState::Materialized)
    );
    Ok(())
}

#[cfg(feature = "duckdb")]
#[test]
#[serial]
fn configured_search_duckdb_connection_opens_in_memory_runtime() -> TestResult {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/runtime-tmp"
threads = 2
preserve_insertion_order = false
parquet_metadata_cache = true
memory_limit = "3GB"
max_temp_directory_size = "11GB"
"#,
    )?;

    let connection =
        crate::duckdb::SearchDuckDbConnection::configured().map_err(std::io::Error::other)?;
    let mut settings = connection
        .connection()
        .prepare(
            "select
                current_setting('threads'),
                current_setting('temp_directory'),
                current_setting('preserve_insertion_order'),
                current_setting('parquet_metadata_cache')
            ",
        )
        .map_err(std::io::Error::other)?;
    let (threads, temp_directory, preserve_insertion_order, parquet_metadata_cache) = settings
        .query_row([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, bool>(3)?,
            ))
        })
        .map_err(std::io::Error::other)?;
    connection
        .connection()
        .execute("create table ducks (name text)", [])
        .map_err(std::io::Error::other)?;
    assert_eq!(threads, 2);
    assert!(
        temp_directory.ends_with(".cache/duckdb/runtime-tmp"),
        "unexpected temp directory setting: {temp_directory}"
    );
    assert!(!preserve_insertion_order);
    assert!(parquet_metadata_cache);

    Ok(())
}

#[cfg(feature = "duckdb")]
#[test]
#[serial]
fn configured_parquet_query_engine_uses_duckdb_in_duckdb_build() -> TestResult {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = false
database_path = ":memory:"
temp_directory = ".cache/duckdb/repo-query-tmp"
threads = 2
"#,
    )?;

    let engine = ParquetQueryEngine::configured().map_err(std::io::Error::other)?;
    assert_eq!(engine.kind(), LocalRelationEngineKind::DuckDb);

    Ok(())
}
