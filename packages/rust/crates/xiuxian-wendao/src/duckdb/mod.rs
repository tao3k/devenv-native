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
mod engine;
mod parquet;
mod runtime;

#[cfg(test)]
#[path = "../../tests/unit/duckdb/mod/mod.rs"]
mod tests;

#[cfg(feature = "duckdb")]
pub use connection::{SearchDuckDbConnection, open_search_duckdb_connection};
#[cfg(feature = "duckdb")]
pub use engine::DuckDbLocalRelationEngine;
#[cfg(all(feature = "duckdb", test))]
pub(crate) use engine::DuckDbRegistrationStrategy;
pub use engine::{
    DataFusionLocalRelationEngine, LocalRelationEngine, LocalRelationEngineKind,
    LocalRelationMaterializationState, LocalRelationRegistrationHint,
};
#[cfg(feature = "duckdb")]
pub use parquet::DuckDbParquetQueryEngine;
pub use parquet::{DataFusionParquetQueryEngine, ParquetQueryEngine};
pub use runtime::resolve_search_duckdb_runtime;
pub use xiuxian_wendao_runtime::config::{
    DuckDbDatabasePath, SearchDuckDbExecutionConfig, SearchDuckDbRuntimeConfig,
};
