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
        let file_paths_set: std::collections::HashSet<String> =
            file_paths.iter().cloned().collect();
        let schema = dataset.schema();
        let has_metadata = schema.field(METADATA_COLUMN).is_some();
        let project_cols: Vec<&str> = if has_metadata {
            vec![ID_COLUMN, crate::FILE_PATH_COLUMN, METADATA_COLUMN]
        } else {
            vec![ID_COLUMN, crate::FILE_PATH_COLUMN]
        };
        let mut scanner = dataset.scan();
        scanner.project(&project_cols)?;
        let mut stream = scanner.try_into_stream().await?;
        let mut ids_to_delete = Vec::new();
        while let Some(batch) = stream.try_next().await? {
            use lance::deps::arrow_array::{Array, StringArray};
            let id_col = batch.column_by_name(ID_COLUMN);
            let file_path_col = batch.column_by_name(crate::FILE_PATH_COLUMN);
            let metadata_col = batch.column_by_name(METADATA_COLUMN);
            let id_arr = id_col.and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let file_path_arr =
                file_path_col.and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let meta_arr = metadata_col.and_then(|c| c.as_any().downcast_ref::<StringArray>());
            if let Some(ids) = id_arr {
                for i in 0..batch.num_rows() {
                    let id = ids.value(i).to_string();
                    let path_from_col = file_path_arr
                        .filter(|arr| !arr.is_null(i))
                        .map(|arr| arr.value(i).to_string())
                        .filter(|s| !s.is_empty());
                    let path = path_from_col.or_else(|| {
                        meta_arr.and_then(|ma| {
                            if ma.is_null(i) {
                                None
                            } else {
                                serde_json::from_str::<serde_json::Value>(ma.value(i))
                                    .ok()
                                    .and_then(|m| {
                                        m.get("file_path")
                                            .and_then(|v| v.as_str())
                                            .map(String::from)
                                    })
                            }
                        })
                    });
                    if let Some(path) = path
                        && file_paths_set.contains(&path)
                    {
                        ids_to_delete.push(id);
                    }
                }
            }
        }
        if !ids_to_delete.is_empty() {
            let escaped: Vec<String> = ids_to_delete
                .iter()
                .map(|id| id.replace('\'', "''"))
                .collect();
            dataset
                .delete(&format!("{ID_COLUMN} IN ('{}')", escaped.join("','")))
                .await?;
        }
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
        let schema = dataset.schema();
        if schema.field(METADATA_COLUMN).is_none() {
            return Ok(0);
        }
        let mut scanner = dataset.scan();
        scanner.project(&[ID_COLUMN, METADATA_COLUMN])?;
        let mut stream = scanner.try_into_stream().await?;
        let mut ids_to_delete = Vec::new();
        while let Some(batch) = stream.try_next().await? {
            use crate::ops::column_read::get_utf8_at;
            use lance::deps::arrow_array::Array;
            let id_col = batch.column_by_name(ID_COLUMN);
            let metadata_col = batch.column_by_name(METADATA_COLUMN);
            let id_arr = id_col.and_then(|c| {
                c.as_any()
                    .downcast_ref::<lance::deps::arrow_array::StringArray>()
            });
            if let Some(ids) = id_arr {
                for i in 0..batch.num_rows() {
                    let id = ids.value(i).to_string();
                    let meta_raw = metadata_col
                        .as_ref()
                        .map(|c| get_utf8_at(c.as_ref(), i))
                        .unwrap_or_default();
                    if meta_raw.is_empty() {
                        continue;
                    }
                    let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_raw) else {
                        continue;
                    };
                    let row_source = meta.get("source").and_then(|v| v.as_str()).unwrap_or("");
                    let matches = row_source == source || row_source.ends_with(source);
                    if matches {
                        ids_to_delete.push(id);
                    }
                }
            }
        }
        let count = u32::try_from(ids_to_delete.len()).unwrap_or(u32::MAX);
        if !ids_to_delete.is_empty() {
            let escaped: Vec<String> = ids_to_delete
                .iter()
                .map(|id| id.replace('\'', "''"))
                .collect();
            dataset
                .delete(&format!("{ID_COLUMN} IN ('{}')", escaped.join("','")))
                .await?;
        }
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
