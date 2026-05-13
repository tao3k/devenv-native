//! Vector index operations: HNSW (IVF+HNSW) and optimal type selection.
//!
//! Phase 3 of the `LanceDB` 2.0 roadmap: high-recall HNSW for smaller tables,
//! and automatic choice of vector index by row count.

use std::time::Instant;

use lance::index::DatasetIndexExt;
use lance::index::vector::VectorIndexParams;
use lance_index::IndexType;
use lance_index::vector::hnsw::builder::HnswBuildParams;
use lance_index::vector::ivf::IvfBuildParams;
use lance_linalg::distance::DistanceType;

use crate::VectorStore;
use crate::error::VectorStoreError;

use super::types::{IndexBuildProgress, IndexStats};

/// Row count below which we use IVF+HNSW (higher recall, more memory).
const HNSW_ROW_THRESHOLD: usize = 10_000;
/// Default IVF partitions for HNSW index.
const HNSW_DEFAULT_PARTITIONS: usize = 64;

impl VectorStore {
    /// Create an IVF+HNSW vector index for higher recall on smaller tables.
    ///
    /// Best for: &lt; 100k vectors, when recall matters more than storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the table is missing, too small, or index creation fails.
    pub async fn create_hnsw_index(
        &self,
        table_name: &str,
    ) -> Result<IndexStats, VectorStoreError> {
        let mut dataset = self.open_hnsw_dataset(table_name).await?;
        let num_rows = hnsw_row_count(&dataset).await?;
        validate_hnsw_row_count(num_rows)?;
        let params = hnsw_index_params(num_rows);

        self.emit_hnsw_index_started(table_name);

        let start = Instant::now();
        dataset
            .create_index(
                &[crate::VECTOR_COLUMN],
                IndexType::Vector,
                None,
                &params,
                true,
            )
            .await
            .map_err(VectorStoreError::LanceDB)?;

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.emit_index_done(duration_ms);

        Ok(IndexStats {
            column: crate::VECTOR_COLUMN.to_string(),
            index_type: "ivf_hnsw".into(),
            duration_ms,
        })
    }

    /// Create the best vector index for the table size (HNSW for small, `IVF_FLAT` for larger).
    ///
    /// # Errors
    ///
    /// Returns an error if table access fails or index creation fails.
    pub async fn create_optimal_vector_index(
        &self,
        table_name: &str,
    ) -> Result<IndexStats, VectorStoreError> {
        let count = self.count(table_name).await? as usize;
        if count < 100 {
            return Err(VectorStoreError::General(
                "create_optimal_vector_index requires at least 100 rows".to_string(),
            ));
        }

        if count < HNSW_ROW_THRESHOLD {
            self.create_hnsw_index(table_name).await
        } else {
            self.create_index(table_name).await?;
            Ok(IndexStats {
                column: crate::VECTOR_COLUMN.to_string(),
                index_type: "ivf_flat".into(),
                duration_ms: 0,
            })
        }
    }
}

impl VectorStore {
    async fn open_hnsw_dataset(
        &self,
        table_name: &str,
    ) -> Result<lance::dataset::Dataset, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Err(VectorStoreError::TableNotFound(table_name.to_string()));
        }
        self.open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await
    }

    fn emit_hnsw_index_started(&self, table_name: &str) {
        if let Some(ref callback) = self.index_progress_callback {
            callback(IndexBuildProgress::Started {
                table_name: table_name.to_string(),
                index_type: "ivf_hnsw".into(),
            });
        }
    }

    fn emit_index_done(&self, duration_ms: u64) {
        if let Some(ref callback) = self.index_progress_callback {
            callback(IndexBuildProgress::Done { duration_ms });
        }
    }
}

async fn hnsw_row_count(dataset: &lance::dataset::Dataset) -> Result<usize, VectorStoreError> {
    dataset
        .count_rows(None)
        .await
        .map_err(VectorStoreError::LanceDB)
}

fn validate_hnsw_row_count(num_rows: usize) -> Result<(), VectorStoreError> {
    if num_rows >= 50 {
        return Ok(());
    }
    Err(VectorStoreError::General(
        "create_hnsw_index requires at least 50 rows".to_string(),
    ))
}

fn hnsw_index_params(num_rows: usize) -> VectorIndexParams {
    let num_partitions = (num_rows / 128).clamp(8, HNSW_DEFAULT_PARTITIONS);
    let ivf = IvfBuildParams::new(num_partitions);
    let hnsw = HnswBuildParams::default();
    VectorIndexParams::ivf_hnsw(DistanceType::L2, ivf, hnsw)
}
