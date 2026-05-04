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
