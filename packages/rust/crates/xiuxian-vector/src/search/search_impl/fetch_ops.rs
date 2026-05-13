use std::collections::BTreeMap;

use futures::TryStreamExt;
use lance::deps::arrow_array::{Array, FixedSizeListArray, Float32Array, StringArray};

use crate::{ID_COLUMN, VECTOR_COLUMN, VectorStore, VectorStoreError};

impl VectorStore {
    /// Fetch stored embedding vectors by document id from one table.
    ///
    /// Missing ids are skipped. Returned keys are unique and sorted because the
    /// output uses a [`BTreeMap`].
    ///
    /// # Errors
    ///
    /// Returns an error when the table cannot be opened, the Lance scanner
    /// cannot be executed, or the table does not expose the expected `id` and
    /// `vector` columns.
    pub async fn fetch_embeddings_by_ids(
        &self,
        table_name: &str,
        ids: &[String],
    ) -> Result<BTreeMap<String, Vec<f32>>, VectorStoreError> {
        if ids.is_empty() {
            return Ok(BTreeMap::new());
        }

        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Err(VectorStoreError::TableNotFound(table_name.to_string()));
        }

        let dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let mut scanner = dataset.scan();
        scanner.project(&[ID_COLUMN, VECTOR_COLUMN])?;
        scanner.filter(build_id_in_filter(ids).as_str())?;

        let mut stream = scanner.try_into_stream().await?;
        let mut embeddings = BTreeMap::new();

        while let Some(batch) = stream.try_next().await? {
            embeddings.extend(fetch_embedding_rows_from_batch(&batch)?);
        }

        Ok(embeddings)
    }
}

struct EmbeddingBatchColumns<'a> {
    ids: &'a StringArray,
    vectors: &'a FixedSizeListArray,
    values: &'a Float32Array,
    vector_len: usize,
}

impl<'a> EmbeddingBatchColumns<'a> {
    fn from_batch(
        batch: &'a lance::deps::arrow_array::RecordBatch,
    ) -> Result<Self, VectorStoreError> {
        let ids = batch
            .column_by_name(ID_COLUMN)
            .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                VectorStoreError::General(format!(
                    "missing Utf8 id column `{ID_COLUMN}` while fetching embeddings"
                ))
            })?;
        let vectors = batch
            .column_by_name(VECTOR_COLUMN)
            .and_then(|column| column.as_any().downcast_ref::<FixedSizeListArray>())
            .ok_or_else(|| {
                VectorStoreError::General(format!(
                    "missing FixedSizeList vector column `{VECTOR_COLUMN}` while fetching embeddings"
                ))
            })?;
        let values = vectors
            .values()
            .as_any()
            .downcast_ref::<Float32Array>()
            .ok_or_else(|| {
                VectorStoreError::General(format!(
                    "vector column `{VECTOR_COLUMN}` does not store Float32 values"
                ))
            })?;
        Ok(Self {
            ids,
            vectors,
            values,
            vector_len: usize::try_from(vectors.value_length()).unwrap_or(0),
        })
    }

    fn row(&self, row: usize) -> Option<(String, Vec<f32>)> {
        if self.ids.is_null(row) || self.vectors.is_null(row) || self.vector_len == 0 {
            return None;
        }
        let range = self.vector_range(row)?;
        let vector = range.map(|index| self.values.value(index)).collect();
        Some((self.ids.value(row).to_string(), vector))
    }

    fn vector_range(&self, row: usize) -> Option<std::ops::Range<usize>> {
        let start = row.saturating_mul(self.vector_len);
        let end = start.saturating_add(self.vector_len);
        (end <= self.values.len()).then_some(start..end)
    }
}

fn fetch_embedding_rows_from_batch(
    batch: &lance::deps::arrow_array::RecordBatch,
) -> Result<BTreeMap<String, Vec<f32>>, VectorStoreError> {
    let columns = EmbeddingBatchColumns::from_batch(batch)?;
    Ok((0..batch.num_rows())
        .filter_map(|row| columns.row(row))
        .collect())
}

fn build_id_in_filter(ids: &[String]) -> String {
    let escaped = ids
        .iter()
        .map(|id| id.replace('\'', "''"))
        .collect::<Vec<_>>();
    format!("{ID_COLUMN} IN ('{}')", escaped.join("','"))
}

#[cfg(test)]
#[path = "../../../tests/unit/search/search_impl/fetch_ops.rs"]
mod tests;
