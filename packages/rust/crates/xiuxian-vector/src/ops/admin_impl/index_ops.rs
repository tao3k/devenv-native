use super::{
    BuiltinIndexType, DistanceType, IndexType, InvertedIndexParams, Result, ScalarIndexParams,
    ScalarIndexType, VECTOR_COLUMN, VectorIndexParams, VectorStore, VectorStoreError,
    index_type_name, open_uri_for_background,
};
use lance::index::DatasetIndexExt;

impl VectorStore {
    /// Create a vector index for a table to optimize search performance.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if dataset opening, row counting, or index creation fails.
    pub async fn create_index(&self, table_name: &str) -> Result<(), VectorStoreError> {
        let table_path = self.table_path(table_name);
        // If table doesn't exist yet, this is a no-op (table will be created when first adding data)
        if !table_path.exists() {
            return Ok(());
        }

        let mut dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let num_rows = dataset
            .count_rows(None)
            .await
            .map_err(VectorStoreError::LanceDB)?;

        // Skip indexing for very small datasets
        if num_rows < 100 {
            return Ok(());
        }

        let num_partitions = (num_rows / 256).clamp(32, 512);
        let params = VectorIndexParams::ivf_flat(num_partitions, DistanceType::L2);

        dataset
            .create_index(&[VECTOR_COLUMN], IndexType::Vector, None, &params, true)
            .await
            .map_err(VectorStoreError::LanceDB)?;
        self.invalidate_cached_table(table_name).await;
        Ok(())
    }

    /// Start building the vector index in a background task. Returns immediately; the index
    /// will be built asynchronously. Errors are logged; use `create_index` if you need to await.
    /// Uses only Lance Dataset in the task (Send-safe; no VectorStore/RefCell across threads).
    pub fn create_index_background(&self, table_name: &str) {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return;
        }
        let uri = table_path.to_string_lossy().into_owned();
        let index_cache_size_bytes = self.index_cache_size_bytes;
        tokio::spawn(async move {
            let mut dataset = match open_uri_for_background(&uri, index_cache_size_bytes).await {
                Ok(ds) => ds,
                Err(e) => {
                    log::warn!("create_index_background: open dataset failed: {e}");
                    return;
                }
            };
            let num_rows = match dataset.count_rows(None).await {
                Ok(n) => n,
                Err(e) => {
                    log::warn!("create_index_background: count_rows failed: {e}");
                    return;
                }
            };
            if num_rows < 100 {
                return;
            }
            let num_partitions = (num_rows / 256).clamp(32, 512);
            let params = VectorIndexParams::ivf_flat(num_partitions, DistanceType::L2);
            if let Err(e) = dataset
                .create_index(
                    &[crate::VECTOR_COLUMN],
                    IndexType::Vector,
                    None,
                    &params,
                    true,
                )
                .await
            {
                log::warn!("create_index_background failed: {e}");
            }
        });
    }

    /// Create a scalar index on a column for fast exact/categorical filtering.
    /// Use `BTree` for equality/range (e.g. `skill_name`), Bitmap for low-cardinality (e.g. category),
    /// and Inverted for generic text or array columns. The table must already have the column.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if dataset opening or scalar index creation fails.
    pub async fn create_scalar_index(
        &self,
        table_name: &str,
        column: &str,
        index_type: ScalarIndexType,
    ) -> Result<(), VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Ok(());
        }
        let mut dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let index_name = format!("scalar_{}_{}", column, index_type_name(index_type));
        match index_type {
            ScalarIndexType::BTree => {
                let params = ScalarIndexParams::for_builtin(BuiltinIndexType::BTree);
                dataset
                    .create_index(&[column], IndexType::BTree, Some(index_name), &params, true)
                    .await
            }
            ScalarIndexType::Bitmap => {
                let params = ScalarIndexParams::for_builtin(BuiltinIndexType::Bitmap);
                dataset
                    .create_index(
                        &[column],
                        IndexType::Bitmap,
                        Some(index_name),
                        &params,
                        true,
                    )
                    .await
            }
            ScalarIndexType::Inverted => {
                let params = InvertedIndexParams::default();
                dataset
                    .create_index(
                        &[column],
                        IndexType::Inverted,
                        Some(index_name),
                        &params,
                        true,
                    )
                    .await
            }
        }
        .map_err(VectorStoreError::LanceDB)?;
        self.invalidate_cached_table(table_name).await;
        Ok(())
    }
}
