//! Cargo entry point for `xiuxian-db-store` unit tests.

#[cfg(feature = "artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ARTIFACT_CACHE_BACKEND_ENV, ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV, ARTIFACT_CACHE_FLUSHERS_ENV,
    ARTIFACT_CACHE_MEMORY_BYTES_ENV, ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
    ARTIFACT_CACHE_RECLAIMERS_ENV, ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV, ARTIFACT_CACHE_ROOT_ENV,
    ARTIFACT_CACHE_RUNTIME_WORKERS_ENV, ARTIFACT_CACHE_STORAGE_BYTES_ENV, AgentArtifactKeyParts,
    ArtifactBlobCache, ArtifactBlobCacheBackendConfig, ArtifactBlobRead, ArtifactBlobReadStatus,
    ArtifactBlobWrite, ArtifactBlobWriteOutcome, ArtifactCacheBackendKind, ArtifactCacheError,
    ArtifactKey, ArtifactKeyComponent, ArtifactKeyParts, ArtifactKind, ArtifactReadThroughStatus,
    AttachmentArtifactKeyParts, ContentAddressedFilesystemBlobCache, OntologyArtifactKeyParts,
    agent_artifact_key, attachment_artifact_key, fetch_through_artifact_bytes,
    ontology_artifact_key, pack_artifact_directory, read_through_artifact_bytes,
    unpack_artifact_directory,
};
#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ArtifactBlobCacheBackend, FoyerArtifactBlobCache, FoyerArtifactBlobCacheConfig,
};
#[cfg(feature = "arrow-codec")]
#[path = "unit/arrow_schema/mod.rs"]
mod arrow_schema;
#[cfg(feature = "artifact-cache")]
#[path = "unit/artifact_cache/mod.rs"]
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
#[cfg(feature = "project-state")]
use xiuxian_db_store::state::{
    ProjectCacheRootConfig, STATE_STORE_DIR_NAME, STATE_STORE_DUCKDB_FILE_NAME,
    git_utils::{
        discover_git_toplevel_from, project_namespace_from_root, sanitize_project_namespace,
    },
    project_cache_root_from_config,
};
#[cfg(all(feature = "engine", not(feature = "vector-store")))]
#[path = "unit/retrieval.rs"]
mod retrieval;
#[cfg(feature = "project-state")]
#[path = "unit/state/mod.rs"]
mod state;
#[cfg(feature = "valkey")]
#[path = "unit/valkey.rs"]
mod valkey;
