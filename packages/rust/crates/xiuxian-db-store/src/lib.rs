//! Bounded database storage facade and lightweight client-local persistence
//! helpers.
//!
//! This crate is the explicit dependency boundary for storage concerns that
//! should not leak into all callers:
//! - Arrow/DataFusion engine types stay in the lightweight `engine` surface
//! - the heavy Lance-backed `vector-store` surface stays feature-gated
//! - the local `DuckDB` surface keeps type-only config and runtime connection
//!   features split so config crates do not compile `DuckDB` unless needed

#[cfg(any(
    all(feature = "arrow-codec", not(feature = "vector-store")),
    all(feature = "artifact-cache", feature = "arrow-codec")
))]
mod arrow_codec;
#[cfg(feature = "arrow-codec")]
/// Shared Arrow table-schema contract helpers.
pub mod arrow_schema;
#[cfg(feature = "artifact-cache")]
/// Attachment and document extraction artifact cache contracts.
pub mod artifact_cache;
#[cfg(feature = "duckdb-types")]
/// Bounded DuckDB configuration and local connection helpers.
pub mod duckdb;
#[cfg(all(feature = "engine", not(feature = "vector-store")))]
mod engine;
#[cfg(all(feature = "engine", not(feature = "vector-store")))]
mod error;

#[cfg(feature = "duckdb")]
pub use ::duckdb as duckdb_crate;

#[cfg(feature = "qianji-bpmn-workflow-state")]
/// Qianji BPMN workflow-state persistence surface.
pub mod qianji_bpmn;
#[cfg(feature = "project-state")]
/// Unified project-local state path contracts.
pub mod state;
#[cfg(feature = "valkey")]
/// Structured Valkey storage primitives for hot indexes and leases.
pub mod valkey;

#[cfg(all(feature = "engine", not(feature = "vector-store")))]
pub use arrow::array::builder::{
    ListBuilder as LanceListBuilder, StringBuilder as LanceStringBuilder,
};
#[cfg(all(feature = "engine", not(feature = "vector-store")))]
pub use arrow::array::{
    Array as LanceArray, ArrayRef as LanceArrayRef, BooleanArray as LanceBooleanArray,
    FixedSizeListArray as LanceFixedSizeListArray, Float32Array as LanceFloat32Array,
    Float64Array as LanceFloat64Array, Int32Array as LanceInt32Array, ListArray as LanceListArray,
    RecordBatch as LanceRecordBatch, StringArray as LanceStringArray,
    UInt32Array as LanceUInt32Array, UInt64Array as LanceUInt64Array,
};
#[cfg(all(feature = "engine", not(feature = "vector-store")))]
pub use arrow::datatypes::{DataType as LanceDataType, Field as LanceField, Schema as LanceSchema};
#[cfg(feature = "engine")]
pub use arrow::record_batch::RecordBatch as EngineRecordBatch;
#[cfg(all(feature = "arrow-codec", not(feature = "vector-store")))]
pub use arrow_codec::{
    attach_record_batch_metadata, attach_record_batch_trace_id, decode_record_batches_ipc,
    encode_record_batch_ipc, encode_record_batches_ipc,
};
#[cfg(all(feature = "artifact-cache", feature = "arrow-codec"))]
pub use arrow_codec::{read_record_batches_ipc_artifact, write_record_batches_ipc_artifact};
#[cfg(feature = "arrow-codec")]
pub use arrow_schema::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaContractError, ArrowSchemaDataType,
    ArrowSchemaNullabilityPolicy, ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY,
    arrow_field_for_column, arrow_fields_for_contract, build_arrow_schema,
    validate_arrow_ipc_stream, validate_arrow_ipc_stream_with_options,
    validate_record_batch_schema, validate_record_batch_schema_with_options,
    validate_schema_against_contract, validate_schema_against_contract_with_options,
};
#[cfg(all(feature = "engine", not(feature = "vector-store")))]
pub use engine::{
    RETRIEVAL_BEST_SECTION_COLUMN, RETRIEVAL_DOC_TYPE_COLUMN, RETRIEVAL_ID_COLUMN,
    RETRIEVAL_LANGUAGE_COLUMN, RETRIEVAL_LINE_COLUMN, RETRIEVAL_MATCH_REASON_COLUMN,
    RETRIEVAL_PATH_COLUMN, RETRIEVAL_REPO_COLUMN, RETRIEVAL_SCORE_COLUMN, RETRIEVAL_SNIPPET_COLUMN,
    RETRIEVAL_SOURCE_COLUMN, RETRIEVAL_TITLE_COLUMN, RetrievalDocType, RetrievalRow,
    payload_fetch_record_batch, retrieval_result_columns, retrieval_result_schema,
    retrieval_rows_from_record_batch, retrieval_rows_to_record_batch,
};
#[cfg(all(feature = "engine", not(feature = "vector-store")))]
pub use engine::{
    SearchEngineContext, SearchEnginePartitionColumn, engine_batch_to_lance_batch,
    engine_batches_to_lance_batches, lance_batch_to_engine_batch, lance_batches_to_engine_batches,
    write_engine_batches_to_parquet_file, write_lance_batches_to_parquet_file,
};
#[cfg(all(feature = "engine", not(feature = "vector-store")))]
pub use error::VectorStoreError;
#[cfg(feature = "valkey")]
pub use valkey::{
    ValkeyClient, ValkeyKeyNamespace, ValkeyLeaseId, ValkeyLeaseOwnership, ValkeyLeaseScriptResult,
    ValkeyQueueEntryId, ValkeyQueueKeys, ValkeyStoreConfig, ValkeyStoreError,
    ValkeyStructuredClaimFilter, ValkeyStructuredClaimRequest, ValkeyStructuredQueue,
    ValkeyStructuredQueueEntry, ValkeyStructuredQueueLease, ValkeyStructuredQueueLeaseRef,
    ValkeyWorkerId,
};

#[cfg(feature = "vector-store")]
pub use xiuxian_vector::{
    CATEGORY_COLUMN, CONTENT_COLUMN, ColumnarScanOptions, CompactionStats, DEFAULT_DIMENSION,
    FILE_PATH_COLUMN, FragmentInfo, ID_COLUMN, INTENTS_COLUMN, IndexBuildProgress, IndexStats,
    IndexStatus, IndexThresholds, LanceArray, LanceArrayRef, LanceBooleanArray, LanceDataType,
    LanceField, LanceFixedSizeListArray, LanceFloat32Array, LanceFloat64Array, LanceInt32Array,
    LanceListArray, LanceListBuilder, LanceRecordBatch, LanceSchema, LanceStringArray,
    LanceStringBuilder, LanceUInt32Array, LanceUInt64Array, METADATA_COLUMN, MergeInsertStats,
    MigrateResult, MigrationItem, QueryMetricsCell, RETRIEVAL_BEST_SECTION_COLUMN,
    RETRIEVAL_DOC_TYPE_COLUMN, RETRIEVAL_ID_COLUMN, RETRIEVAL_LANGUAGE_COLUMN,
    RETRIEVAL_LINE_COLUMN, RETRIEVAL_MATCH_REASON_COLUMN, RETRIEVAL_PATH_COLUMN,
    RETRIEVAL_REPO_COLUMN, RETRIEVAL_SCORE_COLUMN, RETRIEVAL_SNIPPET_COLUMN,
    RETRIEVAL_SOURCE_COLUMN, RETRIEVAL_TITLE_COLUMN, ROUTING_KEYWORDS_COLUMN, Recommendation,
    RetrievalRow, SKILL_NAME_COLUMN, SearchEngineContext, SearchEnginePartitionColumn,
    SearchOptions, THREAD_ID_COLUMN, TOOL_NAME_COLUMN, TableColumnAlteration, TableColumnType,
    TableHealthReport, TableInfo, TableNewColumn, TableVersionInfo, VECTOR_COLUMN,
    VectorRecordBatchReader, VectorStore, VectorStoreError, XIUXIAN_SCHEMA_VERSION,
    attach_record_batch_metadata, attach_record_batch_trace_id, decode_record_batches_ipc,
    encode_record_batch_ipc, encode_record_batches_ipc, json_to_lance_where,
    payload_fetch_record_batch, retrieval_result_columns, retrieval_result_schema,
    retrieval_rows_from_record_batch, retrieval_rows_to_record_batch, schema_version_from_schema,
    string_contains_mask,
};

#[cfg(feature = "vector-store")]
pub use xiuxian_vector::{
    engine_batch_to_lance_batch, engine_batches_to_lance_batches, lance_batch_to_engine_batch,
    lance_batches_to_engine_batches,
};

#[cfg(feature = "vector-store")]
pub use xiuxian_vector::{
    write_engine_batches_to_parquet_file, write_lance_batches_to_parquet_file,
};

#[cfg(all(feature = "engine", not(feature = "vector-store")))]
pub use engine::{ColumnarScanOptions, TableInfo, VectorStore};
