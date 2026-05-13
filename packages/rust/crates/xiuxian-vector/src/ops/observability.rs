//! Observability: table health analysis and recommendations.
//!
//! Phase 5 of the `LanceDB` 2.0 roadmap.
//! Query metrics are in-process (not from Lance tracing yet).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::VectorStore;
use crate::error::VectorStoreError;
use crate::ops::types::{
    IndexCacheStats, IndexStatus, QueryMetrics, Recommendation, TableHealthReport,
};

/// Fragmentation ratio above which we recommend compaction.
const FRAGMENTATION_RATIO_THRESHOLD: f64 = 0.01;
/// Row count above which we recommend having indices.
const ROW_COUNT_INDEX_THRESHOLD: usize = 1000;

impl VectorStore {
    /// Analyze table health and return a report with recommendations.
    ///
    /// # Errors
    ///
    /// Returns an error if metadata queries against the table fail.
    pub async fn analyze_table_health(
        &self,
        table_name: &str,
    ) -> Result<TableHealthReport, VectorStoreError> {
        let row_count = self.count(table_name).await?;
        let fragments = self.get_fragment_stats(table_name).await?;
        let indices = self.describe_indices(table_name).await?;
        let has_vector = self.has_vector_index(table_name).await?;
        let has_scalar = self.has_scalar_index(table_name).await?;
        let fragment_count = fragments.len();
        let fragmentation_ratio = fragmentation_ratio(row_count, fragment_count);
        let indices_status = index_statuses(&indices);
        let recommendations = health_recommendations(
            row_count,
            fragmentation_ratio,
            IndexCoverage {
                has_vector,
                has_scalar,
            },
        );

        Ok(TableHealthReport {
            row_count,
            fragment_count,
            fragmentation_ratio,
            indices_status,
            recommendations,
        })
    }

    /// Record a query for the table (in-process metrics).
    pub fn record_query(&self, table_name: &str, elapsed_ms: u64) {
        let cell = self
            .query_metrics
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(table_name.to_string())
            .or_insert_with(|| Arc::new((AtomicU64::new(0), AtomicU64::new(0))))
            .clone();
        cell.0.fetch_add(1, Ordering::Relaxed);
        cell.1.store(elapsed_ms, Ordering::Relaxed);
    }

    /// Return per-table query metrics. In-process counts and last latency today;
    /// when Lance provides per-query tracing, this can be wired to that instead.
    ///
    #[must_use]
    pub fn get_query_metrics(&self, table_name: &str) -> QueryMetrics {
        let cell = self
            .query_metrics
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(table_name)
            .cloned();
        if let Some(cell) = cell {
            let count = cell.0.load(Ordering::Relaxed);
            let last_ms = cell.1.load(Ordering::Relaxed);
            QueryMetrics {
                query_count: count,
                last_query_ms: if last_ms == 0 { None } else { Some(last_ms) },
            }
        } else {
            QueryMetrics::default()
        }
    }

    /// Return index cache stats (entry count and hit rate) for the table's dataset.
    ///
    /// # Errors
    ///
    /// Returns an error if the dataset cannot be opened.
    pub async fn get_index_cache_stats(
        &self,
        table_name: &str,
    ) -> Result<IndexCacheStats, VectorStoreError> {
        let path = self.table_path(table_name);
        let uri = path.to_string_lossy();
        let dataset = self.open_dataset_at_uri(uri.as_ref()).await?;
        let entry_count = dataset.index_cache_entry_count().await;
        let hit_rate = dataset.index_cache_hit_rate().await;
        Ok(IndexCacheStats {
            entry_count,
            hit_rate,
        })
    }
}

#[derive(Clone, Copy)]
struct IndexCoverage {
    has_vector: bool,
    has_scalar: bool,
}

fn fragmentation_ratio(row_count: u32, fragment_count: usize) -> f64 {
    let total_rows = f64::from(row_count);
    if total_rows == 0.0 {
        return 0.0;
    }
    let fragment_count_f64 = u32::try_from(fragment_count).map_or(f64::from(u32::MAX), f64::from);
    fragment_count_f64 / total_rows
}

fn index_statuses(indices: &[Arc<dyn lance_index::IndexDescription>]) -> Vec<IndexStatus> {
    indices
        .iter()
        .map(|description| IndexStatus {
            name: description.name().to_string(),
            index_type: description.index_type().to_string().into(),
        })
        .collect()
}

fn health_recommendations(
    row_count: u32,
    fragmentation_ratio: f64,
    coverage: IndexCoverage,
) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();
    if fragmentation_ratio > FRAGMENTATION_RATIO_THRESHOLD {
        recommendations.push(Recommendation::RunCompaction);
    }
    if needs_indices(row_count, coverage) {
        recommendations.push(Recommendation::CreateIndices);
    }
    if recommendations.is_empty() {
        recommendations.push(Recommendation::None);
    }
    recommendations
}

fn needs_indices(row_count: u32, coverage: IndexCoverage) -> bool {
    row_count as usize >= ROW_COUNT_INDEX_THRESHOLD
        && (!coverage.has_vector || !coverage.has_scalar)
}
