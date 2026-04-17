#[cfg(feature = "duckdb")]
use std::collections::BTreeMap;
use std::path::Path;
#[cfg(feature = "duckdb")]
use std::path::PathBuf;
#[cfg(feature = "duckdb")]
use std::sync::{Arc, Mutex, MutexGuard};

use xiuxian_vector::{EngineRecordBatch, SearchEngineContext, VectorStoreError};

#[cfg(feature = "duckdb")]
use super::connection::SearchDuckDbConnection;
use super::engine::LocalRelationEngineKind;
#[cfg(feature = "duckdb")]
use super::engine::build_duckdb_parquet_view_sql;
#[cfg(feature = "duckdb")]
use super::runtime::resolve_search_duckdb_runtime;
#[cfg(feature = "duckdb")]
use xiuxian_wendao_runtime::config::SearchDuckDbRuntimeConfig;

/// DataFusion-backed repo publication Parquet query engine.
#[derive(Clone)]
pub struct DataFusionParquetQueryEngine {
    context: SearchEngineContext,
}

impl DataFusionParquetQueryEngine {
    /// Wrap one existing `DataFusion` search-engine context for Parquet reads.
    #[must_use]
    pub fn new(context: SearchEngineContext) -> Self {
        Self { context }
    }
}

/// DuckDB-backed repo publication Parquet query engine.
#[cfg(feature = "duckdb")]
pub struct DuckDbParquetQueryEngine {
    runtime: Mutex<DuckDbParquetRuntime>,
}

#[cfg(feature = "duckdb")]
struct DuckDbParquetRuntime {
    connection: SearchDuckDbConnection,
    registered_parquet_views: BTreeMap<String, PathBuf>,
}

#[cfg(feature = "duckdb")]
impl DuckDbParquetQueryEngine {
    /// Open a `DuckDB`-backed Parquet query engine from one resolved runtime.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured `DuckDB` connection cannot be
    /// initialized.
    pub fn from_runtime(runtime: SearchDuckDbRuntimeConfig) -> Result<Self, VectorStoreError> {
        let connection =
            SearchDuckDbConnection::from_runtime(runtime).map_err(VectorStoreError::General)?;
        Ok(Self {
            runtime: Mutex::new(DuckDbParquetRuntime {
                connection,
                registered_parquet_views: BTreeMap::new(),
            }),
        })
    }

    fn lock_runtime(&self) -> Result<MutexGuard<'_, DuckDbParquetRuntime>, VectorStoreError> {
        self.runtime.lock().map_err(|_| {
            VectorStoreError::General("search DuckDB connection mutex is poisoned".to_string())
        })
    }
}

/// Narrow Parquet-backed query-engine seam for published Parquet search reads.
#[derive(Clone)]
pub enum ParquetQueryEngine {
    /// Execute repo-backed Parquet reads through the existing `DataFusion` lane.
    DataFusion(DataFusionParquetQueryEngine),
    #[cfg(feature = "duckdb")]
    /// Execute repo-backed Parquet reads through the local `DuckDB` lane.
    DuckDb(Arc<DuckDbParquetQueryEngine>),
}

impl ParquetQueryEngine {
    /// Build one configured Parquet query engine for published Parquet reads.
    ///
    /// In `duckdb` builds, routed published-Parquet reads are now explicitly
    /// `DuckDB`-owned and no longer accept a production `DataFusion` fallback
    /// context.
    ///
    /// # Errors
    ///
    /// Returns an error when the resolved `DuckDB` runtime cannot be
    /// initialized.
    #[cfg(feature = "duckdb")]
    pub fn configured() -> Result<Self, VectorStoreError> {
        let mut runtime = resolve_search_duckdb_runtime();
        runtime.enabled = true;
        DuckDbParquetQueryEngine::from_runtime(runtime).map(|engine| Self::DuckDb(Arc::new(engine)))
    }

    /// Build one configured Parquet query engine for repo-backed reads.
    ///
    /// Without the `duckdb` feature compiled in, the query engine always uses
    /// the current `DataFusion` backend.
    #[cfg(not(feature = "duckdb"))]
    #[must_use]
    pub fn configured(default_context: SearchEngineContext) -> Self {
        Self::DataFusion(DataFusionParquetQueryEngine::new(default_context))
    }

    /// Build one explicit `DuckDB`-backed Parquet query engine from one runtime.
    ///
    /// # Errors
    ///
    /// Returns an error when the provided `DuckDB` runtime cannot be
    /// initialized.
    #[cfg(feature = "duckdb")]
    pub fn duckdb_from_runtime(
        runtime: SearchDuckDbRuntimeConfig,
    ) -> Result<Self, VectorStoreError> {
        DuckDbParquetQueryEngine::from_runtime(runtime).map(|engine| Self::DuckDb(Arc::new(engine)))
    }

    /// Report the active engine kind.
    #[must_use]
    pub fn kind(&self) -> LocalRelationEngineKind {
        match self {
            Self::DataFusion(_) => LocalRelationEngineKind::DataFusion,
            #[cfg(feature = "duckdb")]
            Self::DuckDb(_) => LocalRelationEngineKind::DuckDb,
        }
    }

    /// Ensure one published Parquet table is queryable through this engine.
    ///
    /// # Errors
    ///
    /// Returns an error when table registration fails.
    pub async fn ensure_parquet_table_registered(
        &self,
        table_name: &str,
        table_path: &Path,
    ) -> Result<(), VectorStoreError> {
        match self {
            Self::DataFusion(engine) => {
                engine
                    .context
                    .ensure_parquet_table_registered(table_name, table_path, &[])
                    .await
            }
            #[cfg(feature = "duckdb")]
            Self::DuckDb(engine) => engine.register_parquet_view(table_name, table_path),
        }
    }

    /// Execute one SQL query and collect Arrow batches.
    ///
    /// # Errors
    ///
    /// Returns an error when planning or execution fails.
    pub async fn query_batches(
        &self,
        sql: &str,
    ) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
        match self {
            Self::DataFusion(engine) => engine.context.sql_batches(sql).await,
            #[cfg(feature = "duckdb")]
            Self::DuckDb(engine) => engine.query_batches(sql),
        }
    }
}

#[cfg(feature = "duckdb")]
impl DuckDbParquetQueryEngine {
    fn register_parquet_view(
        &self,
        table_name: &str,
        table_path: &Path,
    ) -> Result<(), VectorStoreError> {
        let normalized_path = table_path.to_path_buf();
        let mut guard = self.lock_runtime()?;
        if guard
            .registered_parquet_views
            .get(table_name)
            .is_some_and(|existing_path| existing_path == &normalized_path)
        {
            return Ok(());
        }
        let sql = build_duckdb_parquet_view_sql(table_name, table_path)
            .map_err(VectorStoreError::General)?;
        guard
            .connection
            .connection()
            .execute_batch(sql.as_str())
            .map_err(|error| {
                VectorStoreError::General(format!(
                    "failed to register DuckDB repo publication parquet view `{table_name}`: {error}"
                ))
            })?;
        guard
            .registered_parquet_views
            .insert(table_name.to_string(), normalized_path);
        Ok(())
    }

    fn query_batches(&self, sql: &str) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
        let guard = self.lock_runtime()?;
        let mut statement = guard
            .connection
            .connection()
            .prepare_cached(sql)
            .map_err(|error| {
                VectorStoreError::General(format!(
                    "failed to prepare DuckDB repo publication SQL `{sql}`: {error}"
                ))
            })?;
        let batches = statement
            .query_arrow([])
            .map_err(|error| {
                VectorStoreError::General(format!(
                    "DuckDB repo publication SQL execution failed for `{sql}`: {error}"
                ))
            })?
            .collect::<Vec<_>>();
        Ok(batches)
    }
}

#[cfg(all(test, feature = "duckdb"))]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;

    use tempfile::tempdir;
    use xiuxian_vector::{
        LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray,
        write_lance_batches_to_parquet_file,
    };
    use xiuxian_wendao_runtime::config::{
        DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS,
        DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE, DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW,
        DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER,
    };

    use super::DuckDbParquetQueryEngine;
    use crate::duckdb::{
        DuckDbDatabasePath, SearchDuckDbExecutionConfig, SearchDuckDbRuntimeConfig,
    };

    #[test]
    fn repeated_parquet_view_registration_reuses_cached_entry_for_same_path() {
        let temp = tempdir().expect("tempdir should succeed");
        let parquet_path = temp.path().join("bench.parquet");
        write_test_parquet(parquet_path.as_path());
        let engine = DuckDbParquetQueryEngine::from_runtime(in_memory_runtime(temp.path()))
            .expect("duckdb engine should open");

        engine
            .register_parquet_view("bench_docs", parquet_path.as_path())
            .expect("initial parquet view registration should succeed");
        engine
            .register_parquet_view("bench_docs", parquet_path.as_path())
            .expect("repeated parquet view registration should succeed");

        let guard = engine.lock_runtime().expect("runtime lock should succeed");
        assert_eq!(guard.registered_parquet_views.len(), 1);
        assert_eq!(
            guard.registered_parquet_views.get("bench_docs"),
            Some(&parquet_path)
        );
    }

    #[test]
    fn repeated_parquet_queries_return_readable_batches_on_one_registered_view() {
        let temp = tempdir().expect("tempdir should succeed");
        let parquet_path = temp.path().join("bench.parquet");
        write_test_parquet(parquet_path.as_path());
        let engine = DuckDbParquetQueryEngine::from_runtime(in_memory_runtime(temp.path()))
            .expect("duckdb engine should open");
        engine
            .register_parquet_view("bench_docs", parquet_path.as_path())
            .expect("parquet view registration should succeed");

        let first = engine
            .query_batches("select path from bench_docs")
            .expect("first parquet query should succeed");
        let second = engine
            .query_batches("select path from bench_docs")
            .expect("second parquet query should succeed");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_eq!(first[0].num_rows(), 1);
        assert_eq!(second[0].num_rows(), 1);
    }

    fn in_memory_runtime(root: &Path) -> SearchDuckDbRuntimeConfig {
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

    fn write_test_parquet(path: &Path) {
        let schema = Arc::new(LanceSchema::new_with_metadata(
            vec![LanceField::new("path", LanceDataType::Utf8, false)],
            HashMap::from([("domain".to_string(), "bench_docs".to_string())]),
        ));
        let batch = LanceRecordBatch::try_new(
            schema,
            vec![Arc::new(LanceStringArray::from(vec![
                "src/module.jl".to_string(),
            ]))],
        )
        .expect("record batch should build");
        write_lance_batches_to_parquet_file(path, &[batch])
            .expect("parquet fixture should write successfully");
    }
}
