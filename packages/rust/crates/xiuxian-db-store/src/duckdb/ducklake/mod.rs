//! Generic DuckLake primitives over embedded `DuckDB`.

#[cfg(feature = "duckdb")]
mod append;
#[cfg(feature = "duckdb")]
mod attach;
mod catalog;
mod secret;
mod sql;
mod table;

#[cfg(feature = "duckdb")]
pub use append::append_ducklake_record_batches;
#[cfg(feature = "duckdb")]
pub use attach::attach_ducklake;
pub use catalog::{DuckLakeAttachConfig, DuckLakeCatalog, DuckLakeDataPath};
pub use secret::{DuckDbS3SecretConfig, DuckDbS3SecretProvider, build_duckdb_s3_secret_sql};
pub use sql::{
    build_ducklake_attach_sql, build_ducklake_extension_bootstrap_sql, build_ducklake_use_sql,
};
pub use table::DuckLakeTableRef;
