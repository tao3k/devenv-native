//! Cargo entry point for `xiuxian-db-store` unit tests.

#[cfg(feature = "artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobRead, ArtifactBlobWrite, ArtifactKey, ArtifactKeyComponent,
    ArtifactKeyParts, ArtifactKind, ContentAddressedFilesystemBlobCache,
};
#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{FoyerArtifactBlobCache, FoyerArtifactBlobCacheConfig};
#[cfg(feature = "artifact-cache")]
#[path = "unit/artifact_cache.rs"]
mod artifact_cache;
#[cfg(feature = "foyer-artifact-cache")]
#[path = "unit/foyer_artifact_cache.rs"]
mod foyer_artifact_cache;
#[cfg(feature = "duckdb")]
use xiuxian_db_store::duckdb::{
    DuckDbConnection, DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig,
    DuckDbS3SecretConfig, DuckDbS3SecretProvider, DuckLakeAttachConfig, DuckLakeCatalog,
    DuckLakeDataPath, DuckLakeRecordBatchAppender, DuckLakeTableRef,
    append_ducklake_record_batches, attach_ducklake, build_duckdb_parquet_view_sql,
    build_duckdb_s3_secret_sql, build_duckdb_virtual_view_sql, build_ducklake_attach_sql,
    build_ducklake_extension_bootstrap_sql, build_ducklake_use_sql, ensure_duckdb_identifier,
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
