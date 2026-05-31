//! Lance-backed vector-table storage shell for Xiuxian.

// ============================================================================
// Re-exports from xiuxian-lance
// ============================================================================

pub use arrow::record_batch::RecordBatch as EngineRecordBatch;
pub use arrow_codec::{
    attach_record_batch_metadata, attach_record_batch_trace_id, decode_record_batches_ipc,
    encode_record_batch_ipc, encode_record_batches_ipc,
};
#[cfg(feature = "vector-store")]
pub use lance::deps::arrow_array::ListArray as LanceListArray;
#[cfg(feature = "vector-store")]
pub use lance::deps::arrow_array::builder::{
    ListBuilder as LanceListBuilder, StringBuilder as LanceStringBuilder,
};
#[cfg(feature = "vector-store")]
pub use lance::deps::arrow_array::{
    Array as LanceArray, ArrayRef as LanceArrayRef, BooleanArray as LanceBooleanArray,
    FixedSizeListArray as LanceFixedSizeListArray, Float32Array as LanceFloat32Array,
    Float64Array as LanceFloat64Array, Int32Array as LanceInt32Array,
    RecordBatch as LanceRecordBatch, StringArray as LanceStringArray,
    UInt32Array as LanceUInt32Array, UInt64Array as LanceUInt64Array,
};
#[cfg(feature = "vector-store")]
pub use lance::deps::arrow_schema::{
    DataType as LanceDataType, Field as LanceField, Schema as LanceSchema,
};
#[cfg(feature = "vector-store")]
pub use xiuxian_lance::{
    CATEGORY_COLUMN, CONTENT_COLUMN, DEFAULT_DIMENSION, FILE_PATH_COLUMN, ID_COLUMN,
    INTENTS_COLUMN, METADATA_COLUMN, ROUTING_KEYWORDS_COLUMN, SKILL_NAME_COLUMN, THREAD_ID_COLUMN,
    TOOL_NAME_COLUMN, VECTOR_COLUMN, VectorRecordBatchReader, extract_optional_string,
    extract_string,
};

pub use error::VectorStoreError;
#[cfg(feature = "vector-store")]
pub use ops::{
    ColumnarScanOptions, CompactionStats, FragmentInfo, IndexBuildProgress, IndexStats,
    IndexStatus, IndexThresholds, MergeInsertStats, MigrateResult, MigrationItem, Recommendation,
    TableColumnAlteration, TableColumnType, TableHealthReport, TableInfo, TableNewColumn,
    TableVersionInfo, XIUXIAN_SCHEMA_VERSION, schema_version_from_schema, string_contains_mask,
};
pub use query_support::{
    RETRIEVAL_BEST_SECTION_COLUMN, RETRIEVAL_DOC_TYPE_COLUMN, RETRIEVAL_ID_COLUMN,
    RETRIEVAL_LANGUAGE_COLUMN, RETRIEVAL_LINE_COLUMN, RETRIEVAL_MATCH_REASON_COLUMN,
    RETRIEVAL_PATH_COLUMN, RETRIEVAL_REPO_COLUMN, RETRIEVAL_SCORE_COLUMN, RETRIEVAL_SNIPPET_COLUMN,
    RETRIEVAL_SOURCE_COLUMN, RETRIEVAL_TITLE_COLUMN, RetrievalRow, payload_fetch_record_batch,
    retrieval_result_columns, retrieval_result_schema, retrieval_rows_from_record_batch,
    retrieval_rows_to_record_batch,
};
#[cfg(feature = "vector-store")]
pub use search::SearchOptions;
pub use search_engine::{SearchEngineContext, SearchEnginePartitionColumn};
#[cfg(feature = "vector-store")]
pub use search_engine::{
    engine_batch_to_lance_batch, engine_batches_to_lance_batches, lance_batch_to_engine_batch,
    lance_batches_to_engine_batches, write_engine_batches_to_parquet_file,
    write_lance_batches_to_parquet_file,
};
#[cfg(feature = "vector-store")]
pub use search_impl::json_to_lance_where;
#[cfg(feature = "vector-store")]
pub use store::{IndexProgressCallback, QueryMetricsCell, ScalarIndexType, VectorStore};
/// Vector record-batch construction helpers.
#[cfg(feature = "vector-store")]
pub mod batch;
/// Error types surfaced by vector storage and conversion APIs.
pub mod error;
/// Vector index planning and parameter helpers.
#[cfg(feature = "vector-store")]
pub mod index;
/// Table administration, scan, and maintenance operations.
#[cfg(feature = "vector-store")]
pub mod ops;
/// Arrow-native retrieval batch helpers used by Wendao query-core adapters.
pub mod query_support;
/// Lance search execution helpers for vector-store retrieval.
#[cfg(feature = "vector-store")]
pub mod search;
/// Search cache utilities for deterministic retrieval tests and runtime reuse.
#[cfg(feature = "vector-store")]
pub mod search_cache;
/// Engine-neutral Arrow conversion and parquet helpers.
pub mod search_engine;
/// Test-only fixtures for vector-store integration coverage.
#[cfg(feature = "vector-store")]
pub mod test_support;

mod arrow_codec;
#[cfg(feature = "vector-store")]
#[path = "search/search_impl/mod.rs"]
mod search_impl;
#[cfg(feature = "vector-store")]
mod store;
