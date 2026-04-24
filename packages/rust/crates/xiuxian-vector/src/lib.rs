//! Lance-backed vector-table storage shell for Xiuxian.

#[cfg(feature = "vector-store")]
use std::collections::HashMap;
#[cfg(feature = "vector-store")]
use std::path::PathBuf;
#[cfg(feature = "vector-store")]
use std::sync::Arc;
#[cfg(feature = "vector-store")]
use std::sync::RwLock as StdRwLock;
#[cfg(feature = "vector-store")]
use std::sync::atomic::AtomicU64;

#[cfg(feature = "vector-store")]
use anyhow::Result;
#[cfg(feature = "vector-store")]
use lance::dataset::Dataset;
#[cfg(feature = "vector-store")]
use tokio::sync::RwLock;

#[cfg(feature = "vector-store")]
use ops::DatasetCache;
#[cfg(feature = "vector-store")]
use ops::DatasetCacheConfig;

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
pub mod batch;
pub mod error;
#[cfg(feature = "vector-store")]
pub mod index;
#[cfg(feature = "vector-store")]
pub mod ops;
/// Arrow-native retrieval batch helpers used by Wendao query-core adapters.
pub mod query_support;
#[cfg(feature = "vector-store")]
pub mod search;
#[cfg(feature = "vector-store")]
pub mod search_cache;
pub mod search_engine;
#[cfg(feature = "vector-store")]
pub mod test_support;

mod arrow_codec;
#[cfg(feature = "vector-store")]
#[path = "search/search_impl/mod.rs"]
mod search_impl;

xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");

// ============================================================================
// Vector Store Core
// ============================================================================

/// Per-table query metrics (in-process; not persisted). Used by [`crate::ops::observability::get_query_metrics`].
#[cfg(feature = "vector-store")]
pub type QueryMetricsCell = Arc<(AtomicU64, AtomicU64)>; // (query_count, last_query_ms; 0 means None)

/// Callback for index build progress (Started / Progress / Done). Set optionally for polling or UI.
#[cfg(feature = "vector-store")]
pub type IndexProgressCallback = Arc<dyn Fn(crate::ops::IndexBuildProgress) + Send + Sync>;

/// Lance-backed vector-table storage shell.
#[cfg(feature = "vector-store")]
#[derive(Clone)]
pub struct VectorStore {
    base_path: PathBuf,
    datasets: Arc<RwLock<DatasetCache>>,
    dimension: usize,
    /// Optional index cache size in bytes. When set, datasets are opened via `DatasetBuilder`.
    pub index_cache_size_bytes: Option<usize>,
    /// In-process per-table query metrics (`query_count`, `last_query_ms`).
    pub(crate) query_metrics: Arc<StdRwLock<HashMap<String, QueryMetricsCell>>>,
    /// Optional callback for index build progress (Started/Done; Progress when Lance exposes API).
    pub(crate) index_progress_callback: Option<IndexProgressCallback>,
    /// When `base_path` is ":memory:", a unique id so each store uses its own temp subdir (avoids `DatasetAlreadyExists`).
    pub(crate) memory_mode_id: Option<u64>,
}

// ----------------------------------------------------------------------------
// Vector Store Implementations (Included via include!)
// ----------------------------------------------------------------------------

#[cfg(feature = "vector-store")]
include!("ops/core.rs");
#[cfg(feature = "vector-store")]
include!("ops/writer_impl.rs");
#[cfg(feature = "vector-store")]
include!("ops/admin_impl.rs");

#[cfg(feature = "vector-store")]
impl VectorStore {
    /// Check if a metadata value matches the filter conditions.
    #[must_use]
    pub fn matches_filter(metadata: &serde_json::Value, conditions: &serde_json::Value) -> bool {
        match conditions {
            serde_json::Value::Object(obj) => {
                for (key, value) in obj {
                    let meta_value = if key.contains('.') {
                        let parts: Vec<&str> = key.split('.').collect();
                        let mut current = metadata.clone();
                        for part in parts {
                            if let serde_json::Value::Object(map) = current {
                                current = map.get(part).cloned().unwrap_or(serde_json::Value::Null);
                            } else {
                                return false;
                            }
                        }
                        Some(current)
                    } else {
                        metadata.get(key).cloned()
                    };

                    if let Some(meta_val) = meta_value {
                        match (&meta_val, value) {
                            (serde_json::Value::String(mv), serde_json::Value::String(v)) => {
                                if mv != v {
                                    return false;
                                }
                            }
                            (serde_json::Value::Number(mv), serde_json::Value::Number(v)) => {
                                if mv != v {
                                    return false;
                                }
                            }
                            (serde_json::Value::Bool(mv), serde_json::Value::Bool(v)) => {
                                if mv != v {
                                    return false;
                                }
                            }
                            _ => {
                                let meta_str = meta_val.to_string().trim_matches('"').to_string();
                                let value_str = value.to_string().trim_matches('"').to_string();
                                if meta_str != value_str {
                                    return false;
                                }
                            }
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            _ => true,
        }
    }
}
