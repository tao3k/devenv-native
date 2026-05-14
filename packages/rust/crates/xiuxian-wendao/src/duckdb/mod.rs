//! Bounded local relation-engine seam for the DuckDB rollout.
//!
//! Ownership rule:
//! - `xiuxian-wendao-runtime` owns typed host runtime config
//! - `xiuxian-wendao` owns the bounded local relation-engine bridge used by
//!   Wendao-owned analytic lanes
//! - the default bounded analytics path can remain DataFusion-backed while
//!   explicit bounded pilots adopt DuckDB-backed local execution here

#[cfg(feature = "duckdb")]
mod arrow;
#[cfg(feature = "duckdb")]
mod connection;
#[cfg(feature = "duckdb")]
mod dataset_ontology;
mod engine;
/// Public Wendao boundary.
#[cfg(feature = "duckdb")]
pub mod event_lake;
mod parquet;
mod runtime;

#[cfg(test)]
#[path = "../../tests/unit/duckdb/mod/mod.rs"]
mod tests;

#[cfg(feature = "duckdb")]
pub use connection::{SearchDuckDbConnection, open_search_duckdb_connection};
#[cfg(feature = "duckdb")]
pub use dataset_ontology::{
    DatasetOntologyArrowIpcSourceTableSpec, DatasetOntologyDuckDbMaterializer,
    DatasetOntologyRuntimeMaterializationReport, DatasetOntologyRuntimeMaterializationRequest,
    read_dataset_ontology_arrow_ipc_source_table,
};
#[cfg(feature = "duckdb")]
pub use engine::DuckDbLocalRelationEngine;
#[cfg(all(feature = "duckdb", test))]
pub(crate) use engine::DuckDbRegistrationStrategy;
pub use engine::{
    DataFusionLocalRelationEngine, LocalRelationEngine, LocalRelationEngineKind,
    LocalRelationMaterializationState, LocalRelationRegistrationHint,
};
#[cfg(feature = "duckdb")]
pub use event_lake::{
    WENDAO_EVENT_APPEND_DEFAULT_BATCH_ROWS, WENDAO_EVENT_LAKE_DEFAULT_ALIAS,
    WENDAO_EVENT_LAKE_EVENTS_TABLE, WENDAO_EVENT_QUERY_DEFAULT_LIMIT, WENDAO_EVENT_QUERY_MAX_LIMIT,
    WendaoEventLake, WendaoEventLakeAppender, WendaoEventLakeLocalConfig, WendaoEventQuery,
    WendaoEventRecord, WendaoEventTypeCount, append_wendao_event_batches, append_wendao_events,
    build_wendao_event_lake_table_sql, ensure_wendao_event_lake_table,
    query_wendao_event_type_counts, query_wendao_events, validate_wendao_event_batch,
    wendao_event_record_batch, wendao_event_schema,
};
#[cfg(feature = "duckdb")]
pub use parquet::DuckDbParquetQueryEngine;
pub use parquet::{DataFusionParquetQueryEngine, ParquetQueryEngine};
pub use runtime::resolve_search_duckdb_runtime;
pub use xiuxian_wendao_runtime::config::{
    DuckDbDatabasePath, SearchDuckDbExecutionConfig, SearchDuckDbRuntimeConfig,
};
