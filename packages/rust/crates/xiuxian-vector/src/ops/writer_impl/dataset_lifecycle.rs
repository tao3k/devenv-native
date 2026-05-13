use super::{
    Arc, Dataset, MergeInsertStats, Result, VectorStore, VectorStoreError, default_write_params,
    has_lance_data,
};
use lance::dataset::{MergeInsertBuilder, MergeStats, WhenMatched, WhenNotMatched};
use lance::deps::arrow_array::RecordBatchIterator;

type MergeInsertBatchResult =
    Result<lance::deps::arrow_array::RecordBatch, crate::error::ArrowError>;
type MergeInsertSource = Box<RecordBatchIterator<Vec<MergeInsertBatchResult>>>;

impl VectorStore {
    /// Replace all documents in a table with the provided batch atomically
    /// from the caller perspective (drop then write fresh snapshot).
    ///
    /// Robustness: Never drop when batch is empty; avoids leaving table empty on caller error.
    ///
    /// Positional boundary: this public writer API preserves the existing
    /// document component surface used by callers that already batch ids,
    /// vectors, contents, and metadata separately.
    ///
    /// # Errors
    ///
    /// Returns an error when dropping the target table or writing the replacement batch fails.
    pub async fn replace_documents(
        &mut self,
        table_name: &str,
        ids: Vec<String>,
        vectors: Vec<Vec<f32>>,
        contents: Vec<String>,
        metadatas: Vec<String>,
    ) -> Result<(), VectorStoreError> {
        if ids.is_empty() {
            log::warn!(
                "replace_documents: empty batch for {table_name}; skipping to avoid empty table"
            );
            return Ok(());
        }
        self.drop_table(table_name).await?;
        self.add_documents(table_name, ids, vectors, contents, metadatas)
            .await
    }

    /// Merge-insert (upsert) documents using a key column (default use-case: `id`).
    ///
    /// Positional boundary: the method mirrors [`Self::add_documents`] and
    /// exposes the match key explicitly for Lance merge-insert compatibility.
    ///
    /// # Errors
    ///
    /// Returns an error when source batch preparation fails, dataset open/create fails,
    /// or merge-insert execution fails.
    pub async fn merge_insert_documents(
        &self,
        table_name: &str,
        ids: Vec<String>,
        vectors: Vec<Vec<f32>>,
        contents: Vec<String>,
        metadatas: Vec<String>,
        match_on: &str,
    ) -> Result<MergeInsertStats, VectorStoreError> {
        if ids.is_empty() {
            return Ok(MergeInsertStats::default());
        }

        let source = self.merge_insert_source(ids, vectors, contents, metadatas)?;
        let dataset = self.merge_insert_target_dataset(table_name).await?;
        let (updated_dataset, stats) =
            execute_merge_insert(dataset, source, match_on.to_string()).await?;

        {
            let mut cache = self.datasets.write().await;
            cache.insert(table_name.to_string(), updated_dataset.as_ref().clone());
        }

        Ok(MergeInsertStats {
            inserted: stats.num_inserted_rows,
            updated: stats.num_updated_rows,
            deleted: stats.num_deleted_rows,
            attempts: stats.num_attempts,
            bytes_written: stats.bytes_written,
            files_written: stats.num_files_written,
        })
    }

    fn merge_insert_source(
        &self,
        ids: Vec<String>,
        vectors: Vec<Vec<f32>>,
        contents: Vec<String>,
        metadatas: Vec<String>,
    ) -> Result<MergeInsertSource, VectorStoreError> {
        let (schema, batch) = self.build_document_batch(ids, vectors, contents, metadatas)?;
        let source_batches: Vec<Result<_, crate::error::ArrowError>> = vec![Ok(batch)];
        Ok(Box::new(RecordBatchIterator::new(source_batches, schema)))
    }

    async fn merge_insert_target_dataset(
        &self,
        table_name: &str,
    ) -> Result<Dataset, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if table_path.exists() {
            self.open_dataset_at_uri(table_path.to_string_lossy().as_ref())
                .await
        } else {
            Ok(self.get_or_create_dataset(table_name, false, None).await?.0)
        }
    }

    /// Get or create a dataset. When `initial` is `Some((schema, batch))` and the table is
    /// created, that batch is written (full 10-column schema). Returns `(dataset, created)` so
    /// callers can skip appending when `created` is true.
    ///
    /// # Errors
    ///
    /// Returns an error when opening existing data, creating a new dataset,
    /// cleaning stale artifacts, or writing initial batches fails.
    pub async fn get_or_create_dataset(
        &self,
        table_name: &str,
        force_create: bool,
        initial: Option<(
            Arc<lance::deps::arrow_schema::Schema>,
            lance::deps::arrow_array::RecordBatch,
        )>,
    ) -> Result<(Dataset, bool), VectorStoreError> {
        use lance::deps::arrow_array::RecordBatchIterator;

        let table_path = self.table_path(table_name);
        let is_memory_mode = self.base_path.as_os_str() == ":memory:";
        let write_uri = if is_memory_mode {
            let Some(id) = self.memory_mode_id else {
                return Err(VectorStoreError::General(
                    "memory_mode_id missing while in :memory: mode".to_string(),
                ));
            };
            std::env::temp_dir()
                .join("xiuxian_lance")
                .join(format!("{id:016x}"))
                .join(table_name)
                .to_string_lossy()
                .into_owned()
        } else {
            table_path.to_string_lossy().into_owned()
        };
        let write_path = std::path::Path::new(&write_uri);

        {
            let mut cache = self.datasets.write().await;
            if !force_create
                && let Some(cached) = cache.get(table_name)
                && write_path.exists()
            {
                return Ok((cached, false));
            }
        }

        let (dataset, created) = if has_lance_data(write_path) && !force_create {
            (self.open_dataset_at_uri(&write_uri).await?, false)
        } else {
            if write_path.exists() {
                // When write_path == base_path (base_path ends with `.lance`),
                // selectively remove only `LanceDB` artifacts.
                if write_path == self.base_path.as_path() {
                    Self::remove_lance_artifacts(write_path)?;
                } else {
                    std::fs::remove_dir_all(write_path).map_err(VectorStoreError::from)?;
                }
            }
            let (schema, batches) = if let Some((s, batch)) = initial {
                (s, vec![Ok(batch)])
            } else {
                let schema = self.create_schema();
                let empty = lance::deps::arrow_array::RecordBatch::new_empty(schema.clone());
                (schema, vec![Ok(empty)])
            };
            log::info!(
                "Creating new `LanceDB` dataset at {} with dimension {}",
                write_uri,
                self.dimension
            );
            let ds = Dataset::write(
                Box::new(RecordBatchIterator::new(batches, schema)),
                &write_uri,
                Some(default_write_params()),
            )
            .await?;
            (ds, true)
        };

        {
            let mut cache = self.datasets.write().await;
            cache.insert(table_name.to_string(), dataset.clone());
        }
        Ok((dataset, created))
    }
}

async fn execute_merge_insert(
    dataset: Dataset,
    source: MergeInsertSource,
    match_on: String,
) -> Result<(Arc<Dataset>, MergeStats), VectorStoreError> {
    let mut builder = MergeInsertBuilder::try_new(Arc::new(dataset), vec![match_on])?;
    builder
        .when_matched(WhenMatched::UpdateAll)
        .when_not_matched(WhenNotMatched::InsertAll);
    let job = builder.try_build()?;
    job.execute_reader(source).await.map_err(Into::into)
}
