//! Cargo entry point for `xiuxian-db-store` unit tests.

rust_lang_project_harness::rust_project_harness_gate!();

#[cfg(feature = "duckdb")]
use xiuxian_db_store::duckdb::{
    DuckDbConnection, DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig,
    build_duckdb_parquet_view_sql, build_duckdb_virtual_view_sql, ensure_duckdb_identifier,
    open_duckdb_connection,
};

#[cfg(feature = "duckdb")]
#[path = "unit/duckdb/mod.rs"]
mod duckdb;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[cfg(feature = "qianji-bpmn-workflow-state")]
#[path = "unit/qianji_bpmn/mod.rs"]
mod qianji_bpmn;
