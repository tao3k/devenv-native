//! Lance-backed `VectorStore` state and method families.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock as StdRwLock};

use anyhow::Result;
use lance::dataset::Dataset;
use tokio::sync::RwLock;

use crate::ops::{DatasetCache, DatasetCacheConfig};
use crate::{CONTENT_COLUMN, DEFAULT_DIMENSION, ID_COLUMN, VECTOR_COLUMN, VectorStoreError};

/// Per-table query metrics (in-process; not persisted). Used by [`crate::ops::observability::get_query_metrics`].
pub type QueryMetricsCell = Arc<(AtomicU64, AtomicU64)>; // (query_count, last_query_ms; 0 means None)

/// Callback for index build progress (Started / Progress / Done). Set optionally for polling or UI.
pub type IndexProgressCallback = Arc<dyn Fn(crate::ops::IndexBuildProgress) + Send + Sync>;

/// Lance-backed vector-table storage shell.
#[derive(Clone)]
pub struct VectorStore {
    pub(crate) base_path: PathBuf,
    pub(crate) datasets: Arc<RwLock<DatasetCache>>,
    pub(crate) dimension: usize,
    /// Optional index cache size in bytes. When set, datasets are opened via `DatasetBuilder`.
    pub index_cache_size_bytes: Option<usize>,
    /// In-process per-table query metrics (`query_count`, `last_query_ms`).
    pub(crate) query_metrics: Arc<StdRwLock<HashMap<String, QueryMetricsCell>>>,
    /// Optional callback for index build progress (Started/Done; Progress when Lance exposes API).
    pub(crate) index_progress_callback: Option<IndexProgressCallback>,
    /// When `base_path` is ":memory:", a unique id so each store uses its own temp subdir (avoids `DatasetAlreadyExists`).
    pub(crate) memory_mode_id: Option<u64>,
}

include!("ops/core.rs");

#[path = "ops/admin_impl/mod.rs"]
mod admin_impl;
#[path = "ops/writer_impl/mod.rs"]
mod writer_impl;

pub use admin_impl::ScalarIndexType;

impl VectorStore {
    /// Check if a metadata value matches the filter conditions.
    #[must_use]
    pub fn matches_filter(metadata: &serde_json::Value, conditions: &serde_json::Value) -> bool {
        let Some(obj) = conditions.as_object() else {
            return true;
        };

        obj.iter().all(|(key, expected)| {
            metadata_value_for_key(metadata, key)
                .is_some_and(|actual| metadata_values_match(&actual, expected))
        })
    }
}

fn metadata_value_for_key(metadata: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
    if key.contains('.') {
        return nested_metadata_value(metadata, key);
    }
    metadata.get(key).cloned()
}

fn nested_metadata_value(metadata: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
    let mut current = metadata;
    for part in key.split('.') {
        current = current.as_object()?.get(part)?;
    }
    Some(current.clone())
}

fn metadata_values_match(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
        (serde_json::Value::String(actual), serde_json::Value::String(expected)) => {
            actual == expected
        }
        (serde_json::Value::Number(actual), serde_json::Value::Number(expected)) => {
            actual == expected
        }
        (serde_json::Value::Bool(actual), serde_json::Value::Bool(expected)) => actual == expected,
        _ => json_filter_text(actual) == json_filter_text(expected),
    }
}

fn json_filter_text(value: &serde_json::Value) -> String {
    value.to_string().trim_matches('"').to_string()
}
