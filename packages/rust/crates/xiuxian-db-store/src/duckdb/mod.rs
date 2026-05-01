//! Generic `DuckDB` storage primitives.
//!
//! The type/config helpers are available with `duckdb-types`; opening real
//! `DuckDB` connections require the heavier `duckdb` feature.

#[cfg(feature = "duckdb")]
mod connection;
mod runtime;
mod sql;

#[cfg(feature = "duckdb")]
pub use connection::{DuckDbConnection, open_duckdb_connection};
pub use runtime::{DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig};
pub use sql::{
    build_drop_duckdb_registered_relation_sql, build_duckdb_parquet_view_sql,
    build_duckdb_virtual_view_sql, ensure_duckdb_identifier, quoted_duckdb_identifier,
};

#[cfg(all(test, feature = "duckdb"))]
#[path = "../../tests/unit/duckdb/mod.rs"]
mod tests;
