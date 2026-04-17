#[cfg(feature = "duckdb")]
use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
#[cfg(feature = "duckdb")]
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use datafusion::datasource::MemTable;
use datafusion::execution::context::SessionContext;
use datafusion::prelude::SessionConfig;
use xiuxian_vector::{EngineRecordBatch, SearchEngineContext};
#[cfg(feature = "duckdb")]
use xiuxian_wendao_sql::{
    LocalRelationEngine as BoundedSqlLocalRelationEngine,
    LocalRelationEngineKind as BoundedSqlLocalRelationEngineKind,
    LocalRelationMaterializationState as BoundedSqlLocalRelationMaterializationState,
    LocalRelationRegistrationHint as BoundedSqlLocalRelationRegistrationHint,
};

#[cfg(feature = "duckdb")]
use arrow::datatypes::DataType;
#[cfg(feature = "duckdb")]
use duckdb::profiling::ProfilingInfo;
#[cfg(feature = "duckdb")]
use std::path::Path;
#[cfg(feature = "duckdb")]
use std::sync::{Mutex, MutexGuard};
#[cfg(feature = "duckdb")]
use uuid::Uuid;
#[cfg(feature = "duckdb")]
use xiuxian_wendao_runtime::config::SearchDuckDbRuntimeConfig;

#[cfg(feature = "duckdb")]
use super::arrow::{
    DuckDbArrowRelationStore, WENDAO_ARROW_RELATION_FUNCTION_NAME, WendaoArrowRelationVTab,
};
#[cfg(feature = "duckdb")]
use super::connection::SearchDuckDbConnection;

/// Stable internal engine kinds for bounded local relation execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalRelationEngineKind {
    /// Request-scoped `DataFusion` execution.
    DataFusion,
    /// DuckDB-backed execution.
    DuckDb,
}

impl LocalRelationEngineKind {
    /// Stable explain/telemetry label for the active local relation engine.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DataFusion => "datafusion",
            Self::DuckDb => "duckdb",
        }
    }
}

/// Stable bounded local relation materialization states for explain-facing
/// metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalRelationMaterializationState {
    /// The engine materialized the relation into engine-owned table storage.
    Materialized,
    /// The engine kept the relation virtual over caller-owned Arrow batches.
    Virtual,
}

impl LocalRelationMaterializationState {
    /// Stable explain/telemetry label for the relation materialization state.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Materialized => "materialized",
            Self::Virtual => "virtual",
        }
    }
}

/// Narrow caller hint for one request-scoped local relation registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalRelationRegistrationHint {
    /// Use the engine default registration policy.
    Default,
    /// The caller expects to query the same relation multiple times within the
    /// current request-scoped engine lifetime.
    RepeatedUse,
}

/// Narrow local relation-engine seam for bounded in-process analytics.
#[async_trait]
pub trait LocalRelationEngine: Send + Sync {
    /// Report the active bounded local relation-engine kind.
    fn kind(&self) -> LocalRelationEngineKind;

    /// Register one set of in-memory record batches as a queryable table.
    ///
    /// # Errors
    ///
    /// Returns an error when the batches cannot be normalized into a queryable
    /// in-memory table or when registration fails.
    fn register_record_batches(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<EngineRecordBatch>,
    ) -> Result<(), String>;

    /// Register one set of in-memory record batches as a queryable table with
    /// one caller-provided usage hint.
    ///
    /// # Errors
    ///
    /// Returns an error when the batches cannot be normalized into a queryable
    /// in-memory table or when registration fails.
    fn register_record_batches_with_hint(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<EngineRecordBatch>,
        hint: LocalRelationRegistrationHint,
    ) -> Result<(), String> {
        let _ = hint;
        self.register_record_batches(table_name, schema, batches)
    }

    /// Report the registration strategy used for one registered relation when
    /// the engine exposes that detail.
    #[must_use]
    fn relation_registration_strategy(&self, _table_name: &str) -> Option<&'static str> {
        None
    }

    /// Report the materialization state used for one registered relation when
    /// the engine exposes that detail.
    #[must_use]
    fn relation_materialization_state(
        &self,
        _table_name: &str,
    ) -> Option<LocalRelationMaterializationState> {
        None
    }

    /// Report the peak temp-storage bytes observed for the last bounded local
    /// query when the engine exposes that detail.
    #[must_use]
    fn last_query_temp_storage_peak_bytes(&self) -> Option<u64> {
        None
    }

    /// Execute one SQL query and collect Arrow batches.
    ///
    /// # Errors
    ///
    /// Returns an error when planning or execution fails.
    async fn query_batches(&self, sql: &str) -> Result<Vec<EngineRecordBatch>, String>;
}

/// Current active bounded local relation engine backed by request-scoped
/// `DataFusion`.
#[derive(Clone)]
pub struct DataFusionLocalRelationEngine {
    context: SearchEngineContext,
}

impl DataFusionLocalRelationEngine {
    /// Create a new request-scoped `DataFusion` engine with
    /// `information_schema` enabled.
    #[must_use]
    pub fn new_with_information_schema() -> Self {
        let mut config = SessionConfig::new().with_information_schema(true);
        config.options_mut().execution.collect_statistics = true;
        Self {
            context: SearchEngineContext::new_with_config(config),
        }
    }

    /// Access the underlying `SearchEngineContext`.
    #[must_use]
    pub fn context(&self) -> &SearchEngineContext {
        &self.context
    }

    /// Access the underlying `DataFusion` session.
    #[must_use]
    pub fn session(&self) -> &SessionContext {
        self.context.session()
    }
}

/// DuckDB-backed bounded local relation engine for explicit bounded pilots.
#[cfg(feature = "duckdb")]
pub struct DuckDbLocalRelationEngine {
    connection: Mutex<SearchDuckDbConnection>,
    runtime: SearchDuckDbRuntimeConfig,
    arrow_relation_store: DuckDbArrowRelationStore,
    registration_strategies: Mutex<HashMap<String, DuckDbRegistrationStrategy>>,
    last_query_temp_storage_peak_bytes: Mutex<Option<u64>>,
}

/// Chosen `DuckDB` registration strategy for one request-scoped relation.
#[cfg(feature = "duckdb")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DuckDbRegistrationStrategy {
    /// Keep the relation virtual through a request-scoped Arrow view.
    VirtualArrow,
    /// Materialize the relation into a `DuckDB` table through the appender path.
    MaterializedAppender,
}

#[cfg(feature = "duckdb")]
impl DuckDbRegistrationStrategy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VirtualArrow => "virtual_arrow",
            Self::MaterializedAppender => "materialized_appender",
        }
    }
}

#[cfg(feature = "duckdb")]
impl DuckDbLocalRelationEngine {
    /// Open one DuckDB-backed local relation engine from merged Wendao
    /// settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the `DuckDB` runtime is disabled or when the host
    /// connection cannot be opened.
    pub fn configured() -> Result<Self, String> {
        let connection = SearchDuckDbConnection::configured()?;
        Self::from_connection(connection)
    }

    /// Open one DuckDB-backed local relation engine from one resolved runtime
    /// config.
    ///
    /// # Errors
    ///
    /// Returns an error when the host connection cannot be opened.
    pub fn from_runtime(runtime: SearchDuckDbRuntimeConfig) -> Result<Self, String> {
        let connection = SearchDuckDbConnection::from_runtime(runtime)?;
        Self::from_connection(connection)
    }

    fn lock_connection(&self) -> Result<MutexGuard<'_, SearchDuckDbConnection>, String> {
        self.connection
            .lock()
            .map_err(|_| "search DuckDB connection mutex is poisoned".to_string())
    }

    fn lock_registration_strategies(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<String, DuckDbRegistrationStrategy>>, String> {
        self.registration_strategies
            .lock()
            .map_err(|_| "duckdb registration strategy mutex is poisoned".to_string())
    }

    fn lock_last_query_temp_storage_peak_bytes(
        &self,
    ) -> Result<MutexGuard<'_, Option<u64>>, String> {
        self.last_query_temp_storage_peak_bytes
            .lock()
            .map_err(|_| "duckdb temp storage metric mutex is poisoned".to_string())
    }

    fn from_connection(connection: SearchDuckDbConnection) -> Result<Self, String> {
        let runtime = connection.runtime().clone();
        let arrow_relation_store = DuckDbArrowRelationStore::new(Uuid::new_v4().to_string());
        connection
            .connection()
            .register_table_function::<WendaoArrowRelationVTab>(WENDAO_ARROW_RELATION_FUNCTION_NAME)
            .map_err(|error| {
                format!("failed to register Wendao DuckDB Arrow relation table function: {error}")
            })?;
        Ok(Self {
            connection: Mutex::new(connection),
            runtime,
            arrow_relation_store,
            registration_strategies: Mutex::new(HashMap::new()),
            last_query_temp_storage_peak_bytes: Mutex::new(None),
        })
    }

    fn registration_strategy_for_row_count(&self, total_rows: usize) -> DuckDbRegistrationStrategy {
        if self.runtime.execution.prefer_virtual_arrow
            && (total_rows as u64) < self.runtime.materialize_threshold_rows
        {
            DuckDbRegistrationStrategy::VirtualArrow
        } else {
            DuckDbRegistrationStrategy::MaterializedAppender
        }
    }

    fn registration_strategy(
        &self,
        total_rows: usize,
        hint: LocalRelationRegistrationHint,
    ) -> DuckDbRegistrationStrategy {
        match hint {
            LocalRelationRegistrationHint::Default => {
                self.registration_strategy_for_row_count(total_rows)
            }
            LocalRelationRegistrationHint::RepeatedUse => {
                DuckDbRegistrationStrategy::MaterializedAppender
            }
        }
    }

    fn register_virtual_relation(
        &self,
        table_name: &str,
        schema: &SchemaRef,
        batches: Vec<EngineRecordBatch>,
    ) -> Result<(), String> {
        self.arrow_relation_store
            .insert(table_name, Arc::clone(schema), batches)?;
        let guard = self.lock_connection()?;
        guard
            .connection()
            .execute_batch(&build_duckdb_virtual_view_sql(
                table_name,
                self.arrow_relation_store.namespace(),
                WENDAO_ARROW_RELATION_FUNCTION_NAME,
            )?)
            .map_err(|error| {
                format!("failed to create DuckDB virtual relation view `{table_name}`: {error}")
            })?;
        Ok(())
    }

    fn register_materialized_relation(
        &self,
        table_name: &str,
        schema: &SchemaRef,
        batches: Vec<EngineRecordBatch>,
    ) -> Result<(), String> {
        self.arrow_relation_store.remove(table_name)?;
        let create_table_sql = build_duckdb_create_table_sql(table_name, schema)?;
        let guard = self.lock_connection()?;
        let connection = guard.connection();
        connection
            .execute_batch(&create_table_sql)
            .map_err(|error| {
                format!("failed to create DuckDB local relation table `{table_name}`: {error}")
            })?;
        if batches.is_empty() {
            return Ok(());
        }

        let mut appender = connection.appender(table_name).map_err(|error| {
            format!(
                "failed to open DuckDB appender for local relation table `{table_name}`: {error}"
            )
        })?;
        for batch in batches {
            appender.append_record_batch(batch).map_err(|error| {
                format!("failed to append Arrow batch into DuckDB local relation table `{table_name}`: {error}")
            })?;
        }
        appender.flush().map_err(|error| {
            format!(
                "failed to flush DuckDB appender for local relation table `{table_name}`: {error}"
            )
        })?;
        Ok(())
    }

    pub(crate) fn register_parquet_view(
        &self,
        table_name: &str,
        table_path: &Path,
    ) -> Result<(), String> {
        self.arrow_relation_store.remove(table_name)?;
        let sql = build_duckdb_parquet_view_sql(table_name, table_path)?;
        let guard = self.lock_connection()?;
        guard
            .connection()
            .execute_batch(sql.as_str())
            .map_err(|error| {
                format!("failed to register DuckDB parquet view `{table_name}`: {error}")
            })?;
        Ok(())
    }

    pub(crate) fn execute_batch_sql(&self, sql: &str) -> Result<(), String> {
        let guard = self.lock_connection()?;
        guard
            .connection()
            .execute_batch(sql)
            .map_err(|error| format!("failed to execute DuckDB batch SQL `{sql}`: {error}"))?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn registered_strategy(
        &self,
        table_name: &str,
    ) -> Result<Option<DuckDbRegistrationStrategy>, String> {
        Ok(self
            .lock_registration_strategies()?
            .get(table_name)
            .copied())
    }
}

#[cfg(feature = "duckdb")]
impl Drop for DuckDbLocalRelationEngine {
    fn drop(&mut self) {
        let _ = self.arrow_relation_store.clear();
    }
}

#[async_trait]
impl LocalRelationEngine for DataFusionLocalRelationEngine {
    fn kind(&self) -> LocalRelationEngineKind {
        LocalRelationEngineKind::DataFusion
    }

    fn register_record_batches(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<EngineRecordBatch>,
    ) -> Result<(), String> {
        let partitions = if batches.is_empty() {
            vec![Vec::new()]
        } else {
            vec![batches]
        };
        let mem_table = MemTable::try_new(schema, partitions)
            .map_err(|error| format!("failed to build in-memory relation table: {error}"))?;
        let _ = self.session().deregister_table(table_name);
        self.session()
            .register_table(table_name, Arc::new(mem_table))
            .map_err(|error| {
                format!("failed to register local relation table `{table_name}`: {error}")
            })?;
        Ok(())
    }

    fn relation_materialization_state(
        &self,
        _table_name: &str,
    ) -> Option<LocalRelationMaterializationState> {
        Some(LocalRelationMaterializationState::Materialized)
    }

    async fn query_batches(&self, sql: &str) -> Result<Vec<EngineRecordBatch>, String> {
        self.context
            .sql_batches(sql)
            .await
            .map_err(|error| format!("local relation SQL execution failed for `{sql}`: {error}"))
    }
}

#[cfg(feature = "duckdb")]
#[async_trait]
impl LocalRelationEngine for DuckDbLocalRelationEngine {
    fn kind(&self) -> LocalRelationEngineKind {
        LocalRelationEngineKind::DuckDb
    }

    fn relation_registration_strategy(&self, table_name: &str) -> Option<&'static str> {
        self.lock_registration_strategies()
            .ok()
            .and_then(|strategies| strategies.get(table_name).copied())
            .map(DuckDbRegistrationStrategy::as_str)
    }

    fn relation_materialization_state(
        &self,
        table_name: &str,
    ) -> Option<LocalRelationMaterializationState> {
        self.lock_registration_strategies()
            .ok()
            .and_then(|strategies| strategies.get(table_name).copied())
            .map(|strategy| match strategy {
                DuckDbRegistrationStrategy::VirtualArrow => {
                    LocalRelationMaterializationState::Virtual
                }
                DuckDbRegistrationStrategy::MaterializedAppender => {
                    LocalRelationMaterializationState::Materialized
                }
            })
    }

    fn last_query_temp_storage_peak_bytes(&self) -> Option<u64> {
        self.lock_last_query_temp_storage_peak_bytes()
            .ok()
            .and_then(|value| *value)
    }

    fn register_record_batches(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<EngineRecordBatch>,
    ) -> Result<(), String> {
        LocalRelationEngine::register_record_batches_with_hint(
            self,
            table_name,
            schema,
            batches,
            LocalRelationRegistrationHint::Default,
        )
    }

    fn register_record_batches_with_hint(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<EngineRecordBatch>,
        hint: LocalRelationRegistrationHint,
    ) -> Result<(), String> {
        ensure_duckdb_identifier(table_name, "table")?;
        ensure_duckdb_schema_support(&schema)?;
        for batch in &batches {
            if batch.schema().as_ref() != schema.as_ref() {
                return Err(format!(
                    "duckdb local relation table `{table_name}` received a batch with a mismatched schema"
                ));
            }
        }

        let total_rows = batches.iter().map(EngineRecordBatch::num_rows).sum();
        let strategy = self.registration_strategy(total_rows, hint);
        match strategy {
            DuckDbRegistrationStrategy::VirtualArrow => {
                self.register_virtual_relation(table_name, &schema, batches)?;
            }
            DuckDbRegistrationStrategy::MaterializedAppender => {
                self.register_materialized_relation(table_name, &schema, batches)?;
            }
        }
        self.lock_registration_strategies()?
            .insert(table_name.to_string(), strategy);
        Ok(())
    }

    async fn query_batches(&self, sql: &str) -> Result<Vec<EngineRecordBatch>, String> {
        *self.lock_last_query_temp_storage_peak_bytes()? = None;
        let guard = self.lock_connection()?;
        let connection = guard.connection();
        let mut statement = connection.prepare(sql).map_err(|error| {
            format!("failed to prepare DuckDB local relation SQL `{sql}`: {error}")
        })?;
        let batches = statement
            .query_arrow([])
            .map_err(|error| {
                format!("DuckDB local relation SQL execution failed for `{sql}`: {error}")
            })?
            .collect();
        *self.lock_last_query_temp_storage_peak_bytes()? = connection
            .get_profiling_info()
            .as_ref()
            .and_then(peak_temp_storage_bytes_from_profiling);
        Ok(batches)
    }
}

#[cfg(feature = "duckdb")]
#[async_trait]
impl BoundedSqlLocalRelationEngine for DuckDbLocalRelationEngine {
    fn kind(&self) -> BoundedSqlLocalRelationEngineKind {
        match LocalRelationEngine::kind(self) {
            LocalRelationEngineKind::DataFusion => BoundedSqlLocalRelationEngineKind::DataFusion,
            LocalRelationEngineKind::DuckDb => BoundedSqlLocalRelationEngineKind::DuckDb,
        }
    }

    fn register_record_batches(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
    ) -> Result<(), String> {
        LocalRelationEngine::register_record_batches(self, table_name, schema, batches)
    }

    fn register_record_batches_with_hint(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
        hint: BoundedSqlLocalRelationRegistrationHint,
    ) -> Result<(), String> {
        let hint = match hint {
            BoundedSqlLocalRelationRegistrationHint::Default => {
                LocalRelationRegistrationHint::Default
            }
            BoundedSqlLocalRelationRegistrationHint::RepeatedUse => {
                LocalRelationRegistrationHint::RepeatedUse
            }
        };
        LocalRelationEngine::register_record_batches_with_hint(
            self, table_name, schema, batches, hint,
        )
    }

    fn relation_registration_strategy(&self, table_name: &str) -> Option<&'static str> {
        LocalRelationEngine::relation_registration_strategy(self, table_name)
    }

    fn relation_materialization_state(
        &self,
        table_name: &str,
    ) -> Option<BoundedSqlLocalRelationMaterializationState> {
        LocalRelationEngine::relation_materialization_state(self, table_name).map(|state| {
            match state {
                LocalRelationMaterializationState::Materialized => {
                    BoundedSqlLocalRelationMaterializationState::Materialized
                }
                LocalRelationMaterializationState::Virtual => {
                    BoundedSqlLocalRelationMaterializationState::Virtual
                }
            }
        })
    }

    fn last_query_temp_storage_peak_bytes(&self) -> Option<u64> {
        LocalRelationEngine::last_query_temp_storage_peak_bytes(self)
    }

    async fn query_batches(&self, sql: &str) -> Result<Vec<RecordBatch>, String> {
        LocalRelationEngine::query_batches(self, sql).await
    }
}

#[cfg(feature = "duckdb")]
const DUCKDB_SYSTEM_PEAK_TEMP_DIR_SIZE_METRIC: &str = "SYSTEM_PEAK_TEMP_DIR_SIZE";

#[cfg(feature = "duckdb")]
fn peak_temp_storage_bytes_from_profiling(info: &ProfilingInfo) -> Option<u64> {
    profiling_metric_u64(info, DUCKDB_SYSTEM_PEAK_TEMP_DIR_SIZE_METRIC)
}

#[cfg(feature = "duckdb")]
fn profiling_metric_u64(info: &ProfilingInfo, metric_name: &str) -> Option<u64> {
    info.metrics
        .get(metric_name)
        .and_then(|value| parse_profiling_metric_u64(value))
        .or_else(|| {
            info.children
                .iter()
                .find_map(|child| profiling_metric_u64(child, metric_name))
        })
}

#[cfg(feature = "duckdb")]
fn parse_profiling_metric_u64(value: &str) -> Option<u64> {
    let token = value.split_whitespace().next()?;
    token.replace([',', '_'], "").parse().ok()
}

#[cfg(feature = "duckdb")]
fn ensure_duckdb_schema_support(schema: &SchemaRef) -> Result<(), String> {
    if schema.fields().is_empty() {
        return Err("duckdb local relation tables require at least one column".to_string());
    }
    for field in schema.fields() {
        ensure_duckdb_identifier(field.name(), "column")?;
        let _ = duckdb_sql_type(field.data_type()).map_err(|error| {
            format!(
                "unsupported Arrow data type for DuckDB local relation column `{}`: {error}",
                field.name()
            )
        })?;
    }
    Ok(())
}

#[cfg(feature = "duckdb")]
fn build_duckdb_create_table_sql(table_name: &str, schema: &SchemaRef) -> Result<String, String> {
    let quoted_table_name = quoted_duckdb_identifier(table_name);
    let column_definitions = schema
        .fields()
        .iter()
        .map(|field| {
            let sql_type = duckdb_sql_type(field.data_type()).map_err(|error| {
                format!(
                    "unsupported Arrow data type for DuckDB local relation column `{}`: {error}",
                    field.name()
                )
            })?;
            let nullability = if field.is_nullable() { "" } else { " NOT NULL" };
            Ok(format!(
                "{} {}{}",
                quoted_duckdb_identifier(field.name()),
                sql_type,
                nullability
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(format!(
        "{drop_relation_sql}\nCREATE TABLE {quoted_table_name} ({columns});",
        drop_relation_sql = build_drop_duckdb_registered_relation_sql(table_name),
        columns = column_definitions.join(", ")
    ))
}

#[cfg(feature = "duckdb")]
fn build_duckdb_virtual_view_sql(
    table_name: &str,
    namespace: &str,
    function_name: &str,
) -> Result<String, String> {
    ensure_duckdb_identifier(function_name, "function")?;
    let quoted_table_name = quoted_duckdb_identifier(table_name);
    let escaped_namespace = namespace.replace('\'', "''");
    let escaped_table_name = table_name.replace('\'', "''");
    Ok(format!(
        "{drop_relation_sql}\nCREATE TEMP VIEW {quoted_table_name} AS SELECT * FROM {function_name}('{escaped_namespace}', '{escaped_table_name}');",
        drop_relation_sql = build_drop_duckdb_registered_relation_sql(table_name)
    ))
}

#[cfg(feature = "duckdb")]
pub(crate) fn build_duckdb_parquet_view_sql(
    table_name: &str,
    table_path: &Path,
) -> Result<String, String> {
    ensure_duckdb_identifier(table_name, "table")?;
    let quoted_table_name = quoted_duckdb_identifier(table_name);
    let read_path = if table_path.is_dir() {
        table_path.join("*.parquet")
    } else {
        table_path.to_path_buf()
    };
    let escaped_path = read_path.to_string_lossy().replace('\'', "''");
    Ok(format!(
        "{drop_sql}\nCREATE TEMP VIEW {quoted_table_name} AS SELECT * FROM read_parquet('{escaped_path}');",
        drop_sql = build_drop_duckdb_registered_relation_sql(table_name),
    ))
}

#[cfg(feature = "duckdb")]
pub(crate) fn build_drop_duckdb_registered_relation_sql(table_name: &str) -> String {
    let quoted_table_name = quoted_duckdb_identifier(table_name);
    format!("DROP VIEW IF EXISTS {quoted_table_name};\nDROP TABLE IF EXISTS {quoted_table_name};")
}

#[cfg(feature = "duckdb")]
fn duckdb_sql_type(data_type: &DataType) -> Result<&'static str, String> {
    match data_type {
        DataType::Boolean => Ok("BOOLEAN"),
        DataType::Int8 => Ok("TINYINT"),
        DataType::Int16 => Ok("SMALLINT"),
        DataType::Int32 => Ok("INTEGER"),
        DataType::Int64 => Ok("BIGINT"),
        DataType::UInt8 => Ok("UTINYINT"),
        DataType::UInt16 => Ok("USMALLINT"),
        DataType::UInt32 => Ok("UINTEGER"),
        DataType::UInt64 => Ok("UBIGINT"),
        DataType::Float32 => Ok("FLOAT"),
        DataType::Float64 => Ok("DOUBLE"),
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => Ok("TEXT"),
        DataType::Binary | DataType::LargeBinary | DataType::BinaryView => Ok("BLOB"),
        DataType::Date32 => Ok("DATE"),
        DataType::Timestamp(_, Some(_)) => Ok("TIMESTAMPTZ"),
        DataType::Timestamp(_, None) => Ok("TIMESTAMP"),
        other => Err(other.to_string()),
    }
}

#[cfg(feature = "duckdb")]
pub(crate) fn ensure_duckdb_identifier(identifier: &str, label: &str) -> Result<(), String> {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return Err(format!(
            "duckdb local relation {label} identifiers cannot be blank"
        ));
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(format!(
            "duckdb local relation {label} identifiers must start with an ASCII letter or underscore: `{identifier}`"
        ));
    }
    if !chars.all(|character| character.is_ascii_alphanumeric() || character == '_') {
        return Err(format!(
            "duckdb local relation {label} identifiers must only use ASCII letters, digits, or underscores: `{identifier}`"
        ));
    }
    Ok(())
}

#[cfg(feature = "duckdb")]
pub(crate) fn quoted_duckdb_identifier(identifier: &str) -> String {
    format!("\"{identifier}\"")
}
