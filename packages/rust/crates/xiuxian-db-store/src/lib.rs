//! Bounded database storage facade and lightweight client-local persistence
//! helpers.
//!
//! This crate is the explicit dependency boundary for storage concerns that
//! should not leak into all callers:
//! - the heavy Lance-backed `vector-store` surface stays feature-gated
//! - the lightweight local `SQLite` surface stays feature-gated
//!
//! The vector-store feature intentionally exports an explicit storage/Arrow
//! compatibility surface instead of exposing the retiring vector-table shell
//! directly.

#[cfg(feature = "sqlite")]
pub mod sql;

#[cfg(feature = "sqlite")]
pub use rusqlite;

#[cfg(feature = "vector-store")]
pub use xiuxian_vector::{
    CATEGORY_COLUMN, CONTENT_COLUMN, ColumnarScanOptions, CompactionStats, DEFAULT_DIMENSION,
    EngineRecordBatch, FILE_PATH_COLUMN, FragmentInfo, ID_COLUMN, INTENTS_COLUMN,
    IndexBuildProgress, IndexStats, IndexStatus, IndexThresholds, LanceArray, LanceArrayRef,
    LanceBooleanArray, LanceDataType, LanceField, LanceFixedSizeListArray, LanceFloat32Array,
    LanceFloat64Array, LanceInt32Array, LanceListArray, LanceListBuilder, LanceRecordBatch,
    LanceSchema, LanceStringArray, LanceStringBuilder, LanceUInt32Array, LanceUInt64Array,
    METADATA_COLUMN, MergeInsertStats, MigrateResult, MigrationItem, QueryMetricsCell,
    RETRIEVAL_BEST_SECTION_COLUMN, RETRIEVAL_DOC_TYPE_COLUMN, RETRIEVAL_ID_COLUMN,
    RETRIEVAL_LANGUAGE_COLUMN, RETRIEVAL_LINE_COLUMN, RETRIEVAL_MATCH_REASON_COLUMN,
    RETRIEVAL_PATH_COLUMN, RETRIEVAL_REPO_COLUMN, RETRIEVAL_SCORE_COLUMN, RETRIEVAL_SNIPPET_COLUMN,
    RETRIEVAL_SOURCE_COLUMN, RETRIEVAL_TITLE_COLUMN, ROUTING_KEYWORDS_COLUMN, Recommendation,
    RetrievalRow, SKILL_NAME_COLUMN, SearchEngineContext, SearchEnginePartitionColumn,
    SearchOptions, THREAD_ID_COLUMN, TOOL_NAME_COLUMN, TableColumnAlteration, TableColumnType,
    TableHealthReport, TableInfo, TableNewColumn, TableVersionInfo, VECTOR_COLUMN,
    VectorRecordBatchReader, VectorStore, VectorStoreError, XIUXIAN_SCHEMA_VERSION,
    attach_record_batch_metadata, attach_record_batch_trace_id, decode_record_batches_ipc,
    encode_record_batch_ipc, encode_record_batches_ipc, engine_batch_to_lance_batch,
    engine_batches_to_lance_batches, extract_optional_string, extract_string, json_to_lance_where,
    lance_batch_to_engine_batch, lance_batches_to_engine_batches, payload_fetch_record_batch,
    retrieval_result_columns, retrieval_result_schema, retrieval_rows_from_record_batch,
    retrieval_rows_to_record_batch, schema_version_from_schema, string_contains_mask,
    write_engine_batches_to_parquet_file, write_lance_batches_to_parquet_file,
};
