use std::collections::HashSet;

use lance::deps::arrow_array::{Array, RecordBatch, StringArray};

use super::{ID_COLUMN, METADATA_COLUMN, Result, TryStreamExt, VectorStore, VectorStoreError};

impl VectorStore {
    /// Delete records by IDs.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if opening the dataset or issuing delete operations fails.
    pub async fn delete(&self, table_name: &str, ids: Vec<String>) -> Result<(), VectorStoreError> {
        let table_path = self.table_path(table_name);
        // If table doesn't exist, nothing to delete
        if !table_path.exists() {
            return Ok(());
        }
        let mut dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        for id in ids {
            dataset.delete(&format!("{ID_COLUMN} = '{id}'")).await?;
        }
        Ok(())
    }

    /// Delete records associated with specific file paths.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if dataset access, projection setup, streaming,
    /// or delete execution fails.
    pub async fn delete_by_file_path(
        &self,
        table_name: &str,
        file_paths: Vec<String>,
    ) -> Result<(), VectorStoreError> {
        let table_path = self.table_path(table_name);
        // If table doesn't exist, nothing to delete
        if !table_path.exists() {
            return Ok(());
        }
        if file_paths.is_empty() {
            return Ok(());
        }
        let mut dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let file_paths_set = file_paths.into_iter().collect();
        let ids_to_delete = collect_ids_matching_file_paths(&dataset, &file_paths_set).await?;
        delete_ids_from_dataset(&mut dataset, &ids_to_delete).await?;
        Ok(())
    }

    /// Delete records whose metadata.source equals or ends with the given source (e.g. document path).
    /// Used for idempotent ingest: delete existing chunks for a document before re-ingesting.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if dataset access, scan setup, stream decoding,
    /// or delete execution fails.
    pub async fn delete_by_metadata_source(
        &self,
        table_name: &str,
        source: &str,
    ) -> Result<u32, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Ok(0);
        }
        if source.is_empty() {
            return Ok(0);
        }
        let mut dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        if dataset.schema().field(METADATA_COLUMN).is_none() {
            return Ok(0);
        }
        let ids_to_delete = collect_ids_matching_metadata_source(&dataset, source).await?;
        let count = u32::try_from(ids_to_delete.len()).unwrap_or(u32::MAX);
        delete_ids_from_dataset(&mut dataset, &ids_to_delete).await?;
        Ok(count)
    }

    /// Drop a table and remove its data from disk.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if filesystem cleanup fails or memory-mode state is invalid.
    pub async fn drop_table(&mut self, table_name: &str) -> Result<(), VectorStoreError> {
        let table_path = self.table_path(table_name);
        let is_memory_mode = self.base_path.as_os_str() == ":memory:";
        let drop_path = if is_memory_mode {
            let Some(id) = self.memory_mode_id else {
                return Err(VectorStoreError::General(
                    "memory_mode_id missing while in :memory: mode".to_string(),
                ));
            };
            std::env::temp_dir()
                .join("xiuxian_lance")
                .join(format!("{id:016x}"))
                .join(table_name)
        } else {
            table_path.clone()
        };
        {
            let mut cache = self.datasets.write().await;
            cache.remove(table_name);
        }
        if drop_path.exists() {
            if drop_path == self.base_path {
                Self::remove_lance_artifacts(&drop_path)?;
            } else {
                std::fs::remove_dir_all(&drop_path)?;
            }
        }
        Ok(())
    }

    /// Remove only LanceDB-specific artifacts from a directory.
    pub(crate) fn remove_lance_artifacts(dir: &std::path::Path) -> Result<(), VectorStoreError> {
        static LANCE_DIRS: &[&str] = &[
            "_versions",
            "data",
            "_indices",
            "_transactions",
            "_deletions",
        ];
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            // Remove known LanceDB directories and nested table dirs.
            if LANCE_DIRS.contains(&name_str.as_ref()) || name_str.ends_with(".lance") {
                std::fs::remove_dir_all(entry.path())?;
            }
            // Remove loose files (e.g. *.manifest)
            else if entry.file_type()?.is_file() {
                std::fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}

async fn collect_ids_matching_file_paths(
    dataset: &lance::dataset::Dataset,
    file_paths: &HashSet<String>,
) -> Result<Vec<String>, VectorStoreError> {
    let project_cols =
        file_path_delete_projection(dataset.schema().field(METADATA_COLUMN).is_some());
    let mut scanner = dataset.scan();
    scanner.project(&project_cols)?;
    let mut stream = scanner.try_into_stream().await?;
    let mut ids_to_delete = Vec::new();
    while let Some(batch) = stream.try_next().await? {
        ids_to_delete.extend(file_path_delete_ids_from_batch(&batch, file_paths));
    }
    Ok(ids_to_delete)
}

fn file_path_delete_projection(has_metadata: bool) -> Vec<&'static str> {
    if has_metadata {
        vec![ID_COLUMN, crate::FILE_PATH_COLUMN, METADATA_COLUMN]
    } else {
        vec![ID_COLUMN, crate::FILE_PATH_COLUMN]
    }
}

fn file_path_delete_ids_from_batch(
    batch: &RecordBatch,
    file_paths: &HashSet<String>,
) -> Vec<String> {
    let Some(ids) = string_column(batch, ID_COLUMN) else {
        return Vec::new();
    };
    let file_path_arr = string_column(batch, crate::FILE_PATH_COLUMN);
    let metadata_arr = string_column(batch, METADATA_COLUMN);
    (0..batch.num_rows())
        .filter_map(|row| {
            file_path_delete_id_for_row(ids, file_path_arr, metadata_arr, row, file_paths)
        })
        .collect()
}

fn file_path_delete_id_for_row(
    ids: &StringArray,
    file_path_arr: Option<&StringArray>,
    metadata_arr: Option<&StringArray>,
    row: usize,
    file_paths: &HashSet<String>,
) -> Option<String> {
    let path = row_file_path(file_path_arr, metadata_arr, row)?;
    file_paths
        .contains(&path)
        .then(|| ids.value(row).to_string())
}

fn row_file_path(
    file_path_arr: Option<&StringArray>,
    metadata_arr: Option<&StringArray>,
    row: usize,
) -> Option<String> {
    path_from_column(file_path_arr, row).or_else(|| path_from_metadata(metadata_arr, row))
}

fn path_from_column(file_path_arr: Option<&StringArray>, row: usize) -> Option<String> {
    file_path_arr
        .filter(|arr| !arr.is_null(row))
        .map(|arr| arr.value(row).to_string())
        .filter(|path| !path.is_empty())
}

fn path_from_metadata(metadata_arr: Option<&StringArray>, row: usize) -> Option<String> {
    metadata_value(metadata_arr, row).and_then(|metadata| {
        metadata
            .get("file_path")
            .and_then(|value| value.as_str())
            .map(String::from)
    })
}

async fn collect_ids_matching_metadata_source(
    dataset: &lance::dataset::Dataset,
    source: &str,
) -> Result<Vec<String>, VectorStoreError> {
    let mut scanner = dataset.scan();
    scanner.project(&[ID_COLUMN, METADATA_COLUMN])?;
    let mut stream = scanner.try_into_stream().await?;
    let mut ids_to_delete = Vec::new();
    while let Some(batch) = stream.try_next().await? {
        ids_to_delete.extend(metadata_source_delete_ids_from_batch(&batch, source));
    }
    Ok(ids_to_delete)
}

fn metadata_source_delete_ids_from_batch(batch: &RecordBatch, source: &str) -> Vec<String> {
    let Some(ids) = string_column(batch, ID_COLUMN) else {
        return Vec::new();
    };
    let metadata_arr = string_column(batch, METADATA_COLUMN);
    (0..batch.num_rows())
        .filter_map(|row| metadata_source_delete_id_for_row(ids, metadata_arr, row, source))
        .collect()
}

fn metadata_source_delete_id_for_row(
    ids: &StringArray,
    metadata_arr: Option<&StringArray>,
    row: usize,
    source: &str,
) -> Option<String> {
    let metadata = metadata_value(metadata_arr, row)?;
    let row_source = metadata.get("source").and_then(|value| value.as_str())?;
    (row_source == source || row_source.ends_with(source)).then(|| ids.value(row).to_string())
}

fn metadata_value(metadata_arr: Option<&StringArray>, row: usize) -> Option<serde_json::Value> {
    let metadata_arr = metadata_arr.filter(|arr| !arr.is_null(row))?;
    serde_json::from_str(metadata_arr.value(row)).ok()
}

fn string_column<'a>(batch: &'a RecordBatch, column: &str) -> Option<&'a StringArray> {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
}

async fn delete_ids_from_dataset(
    dataset: &mut lance::dataset::Dataset,
    ids_to_delete: &[String],
) -> Result<(), VectorStoreError> {
    if ids_to_delete.is_empty() {
        return Ok(());
    }
    dataset
        .delete(delete_ids_filter(ids_to_delete).as_str())
        .await?;
    Ok(())
}

fn delete_ids_filter(ids_to_delete: &[String]) -> String {
    let escaped = ids_to_delete
        .iter()
        .map(|id| id.replace('\'', "''"))
        .collect::<Vec<_>>();
    format!("{ID_COLUMN} IN ('{}')", escaped.join("','"))
}
