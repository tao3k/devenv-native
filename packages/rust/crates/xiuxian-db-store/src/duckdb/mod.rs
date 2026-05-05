//! Generic `DuckDB` storage primitives.
//!
//! The type/config helpers are available with `duckdb-types`; opening real
//! `DuckDB` connections require the heavier `duckdb` feature.

#[cfg(feature = "duckdb")]
mod connection;
mod ducklake;
mod runtime;
mod sql;

#[cfg(feature = "duckdb")]
pub use connection::{DuckDbConnection, open_duckdb_connection};
pub use ducklake::{
    DuckDbS3SecretConfig, DuckDbS3SecretProvider, DuckLakeAttachConfig, DuckLakeCatalog,
    DuckLakeDataPath, DuckLakeTableRef, build_duckdb_s3_secret_sql, build_ducklake_attach_sql,
    build_ducklake_extension_bootstrap_sql, build_ducklake_use_sql,
};
#[cfg(feature = "duckdb")]
pub use ducklake::{DuckLakeRecordBatchAppender, append_ducklake_record_batches, attach_ducklake};
pub use runtime::{DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig};
pub use sql::{
    build_drop_duckdb_registered_relation_sql, build_duckdb_parquet_view_sql,
    build_duckdb_virtual_view_sql, ensure_duckdb_identifier, quoted_duckdb_identifier,
};

#[cfg(all(test, feature = "duckdb"))]
#[path = "../../tests/unit/duckdb/mod.rs"]
mod tests;
