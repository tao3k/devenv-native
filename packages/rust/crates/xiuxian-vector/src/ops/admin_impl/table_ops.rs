use super::{
    Arc, Dataset, FragmentInfo, ID_COLUMN, METADATA_COLUMN, Result, TableColumnAlteration,
    TableColumnType, TableInfo, TableNewColumn, TableVersionInfo, TryStreamExt, VectorStore,
    VectorStoreError, is_dataset_not_found_or_invalid,
};
use lance::deps::arrow_array::{Array, RecordBatch, StringArray};

impl VectorStore {
    /// Get the number of rows in a table.
    /// Returns 0 if the table path does not exist or the dataset was dropped (e.g. after
    /// `drop_table` when `base_path` ends with `.lance`, the directory may remain but Lance
    /// artifacts like `_versions` are removed).
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if dataset opening or row counting fails
    /// for reasons other than missing/invalid dropped-table artifacts.
    pub async fn count(&self, table_name: &str) -> Result<u32, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Ok(0);
        }
        let dataset = match self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await
        {
            Ok(d) => d,
            Err(e) if is_dataset_not_found_or_invalid(&e) => return Ok(0),
            Err(e) => return Err(e),
        };
        Ok(u32::try_from(dataset.count_rows(None).await?).unwrap_or(0))
    }

    /// Get the latest table version id.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if the table does not exist or version lookup fails.
    pub async fn get_dataset_version(&self, table_name: &str) -> Result<u64, VectorStoreError> {
        let dataset = self.open_table_or_err(table_name).await?;
        dataset.latest_version_id().await.map_err(Into::into)
    }

    /// Open a historical snapshot by version (time travel).
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if table opening fails or the requested version cannot be checked out.
    pub async fn checkout_version(
        &self,
        table_name: &str,
        version: u64,
    ) -> Result<Dataset, VectorStoreError> {
        let dataset = self.open_table_or_err(table_name).await?;
        dataset.checkout_version(version).await.map_err(Into::into)
    }

    /// List all historical versions of a table.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if table opening or version listing fails.
    pub async fn list_versions(
        &self,
        table_name: &str,
    ) -> Result<Vec<TableVersionInfo>, VectorStoreError> {
        let dataset = self.open_table_or_err(table_name).await?;
        let versions = dataset.versions().await?;

        Ok(versions
            .into_iter()
            .map(|version| TableVersionInfo {
                version_id: version.version,
                timestamp: version.timestamp.to_rfc3339(),
                metadata: version.metadata,
            })
            .collect())
    }

    /// Get basic table observability info for dashboard/admin usage.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if table opening or row counting fails.
    pub async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, VectorStoreError> {
        let dataset = self.open_table_or_err(table_name).await?;
        let version = dataset.version();
        let num_rows = dataset.count_rows(None).await?;

        Ok(TableInfo {
            version_id: version.version,
            commit_timestamp: version.timestamp.to_rfc3339(),
            num_rows: num_rows as u64,
            schema: format!("{:?}", dataset.schema()),
            fragment_count: dataset.count_fragments(),
        })
    }

    /// Get fragment-level row/file stats to support query tuning and diagnostics.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if table opening or fragment row counting fails.
    pub async fn get_fragment_stats(
        &self,
        table_name: &str,
    ) -> Result<Vec<FragmentInfo>, VectorStoreError> {
        let dataset = self.open_table_or_err(table_name).await?;
        let mut stats = Vec::new();

        for fragment in dataset.get_fragments() {
            let num_rows = fragment.count_rows(None).await?;
            let metadata = fragment.metadata();
            stats.push(FragmentInfo {
                id: fragment.id(),
                num_rows,
                physical_rows: metadata.physical_rows,
                num_data_files: metadata.files.len(),
            });
        }

        Ok(stats)
    }

    /// Add new columns to a table as schema evolution.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if table opening fails, reserved columns are requested,
    /// or schema update operations fail.
    pub async fn add_columns(
        &self,
        table_name: &str,
        columns: Vec<TableNewColumn>,
    ) -> Result<(), VectorStoreError> {
        use lance::dataset::NewColumnTransform;
        use lance::deps::arrow_schema::{DataType, Field, Schema};

        if columns.is_empty() {
            return Ok(());
        }

        let mut dataset = self.open_table_or_err(table_name).await?;
        let fields = columns
            .into_iter()
            .map(|column| {
                Self::ensure_non_reserved_column(&column.name)?;
                let data_type = match column.data_type {
                    TableColumnType::Utf8 => DataType::Utf8,
                    TableColumnType::Int64 => DataType::Int64,
                    TableColumnType::Float64 => DataType::Float64,
                    TableColumnType::Boolean => DataType::Boolean,
                };
                Ok(Field::new(&column.name, data_type, column.nullable))
            })
            .collect::<Result<Vec<_>, VectorStoreError>>()?;

        let schema = Arc::new(Schema::new(fields));
        dataset
            .add_columns(NewColumnTransform::AllNulls(schema), None, None)
            .await?;
        {
            let mut cache = self.datasets.write().await;
            cache.insert(table_name.to_string(), dataset.clone());
        }
        Ok(())
    }

    /// Apply schema alterations such as rename and nullability changes.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if table opening fails, reserved columns are referenced,
    /// or alteration execution fails.
    pub async fn alter_columns(
        &self,
        table_name: &str,
        alterations: Vec<TableColumnAlteration>,
    ) -> Result<(), VectorStoreError> {
        use lance::dataset::ColumnAlteration as LanceColumnAlteration;

        if alterations.is_empty() {
            return Ok(());
        }

        let mut dataset = self.open_table_or_err(table_name).await?;
        let mut lance_alterations = Vec::with_capacity(alterations.len());

        for alteration in alterations {
            match alteration {
                TableColumnAlteration::Rename { path, new_name } => {
                    Self::ensure_non_reserved_column(&path)?;
                    lance_alterations.push(LanceColumnAlteration::new(path).rename(new_name));
                }
                TableColumnAlteration::SetNullable { path, nullable } => {
                    Self::ensure_non_reserved_column(&path)?;
                    lance_alterations.push(LanceColumnAlteration::new(path).set_nullable(nullable));
                }
            }
        }

        dataset.alter_columns(&lance_alterations).await?;
        {
            let mut cache = self.datasets.write().await;
            cache.insert(table_name.to_string(), dataset.clone());
        }
        Ok(())
    }

    /// Drop non-reserved columns from a table.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if reserved columns are requested,
    /// table opening fails, or column drop execution fails.
    pub async fn drop_columns(
        &self,
        table_name: &str,
        columns: Vec<String>,
    ) -> Result<(), VectorStoreError> {
        if columns.is_empty() {
            return Ok(());
        }
        for column in &columns {
            Self::ensure_non_reserved_column(column)?;
        }

        let mut dataset = self.open_table_or_err(table_name).await?;
        let refs: Vec<&str> = columns.iter().map(String::as_str).collect();
        dataset.drop_columns(&refs).await?;
        {
            let mut cache = self.datasets.write().await;
            cache.insert(table_name.to_string(), dataset.clone());
        }
        Ok(())
    }

    /// Retrieve all file paths and their hashes stored in the table.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreError`] if dataset access, scanning/streaming, or JSON serialization fails.
    pub async fn get_all_file_hashes(&self, table_name: &str) -> Result<String, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Ok("{}".to_string());
        }
        let dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let project_cols = file_hash_projection(dataset.schema().field(METADATA_COLUMN).is_some());
        let mut scanner = dataset.scan();
        scanner.project(&project_cols)?;
        let mut stream = scanner.try_into_stream().await?;
        let mut hash_map = std::collections::HashMap::new();
        while let Some(batch) = stream.try_next().await? {
            hash_map.extend(file_hash_rows_from_batch(&batch));
        }
        serde_json::to_string(&hash_map).map_err(|e| VectorStoreError::General(e.to_string()))
    }
}

fn file_hash_projection(has_metadata: bool) -> Vec<&'static str> {
    if has_metadata {
        vec![ID_COLUMN, crate::FILE_PATH_COLUMN, METADATA_COLUMN]
    } else {
        vec![ID_COLUMN, crate::FILE_PATH_COLUMN]
    }
}

fn file_hash_rows_from_batch(
    batch: &RecordBatch,
) -> std::collections::HashMap<String, serde_json::Value> {
    let Some(ids) = string_column(batch, ID_COLUMN) else {
        return std::collections::HashMap::new();
    };
    let file_path_arr = string_column(batch, crate::FILE_PATH_COLUMN);
    let metadata_arr = string_column(batch, METADATA_COLUMN);
    (0..batch.num_rows())
        .filter_map(|row| file_hash_row(ids, file_path_arr, metadata_arr, row))
        .collect()
}

fn file_hash_row(
    ids: &StringArray,
    file_path_arr: Option<&StringArray>,
    metadata_arr: Option<&StringArray>,
    row: usize,
) -> Option<(String, serde_json::Value)> {
    let metadata = metadata_value(metadata_arr, row);
    let path = path_from_column(file_path_arr, row)
        .or_else(|| metadata.as_ref().and_then(path_from_metadata))?;
    let hash = metadata
        .as_ref()
        .and_then(hash_from_metadata)
        .unwrap_or_default();
    Some((
        path,
        serde_json::json!({ "hash": hash, "id": ids.value(row).to_string() }),
    ))
}

fn path_from_column(file_path_arr: Option<&StringArray>, row: usize) -> Option<String> {
    file_path_arr
        .filter(|arr| !arr.is_null(row))
        .map(|arr| arr.value(row).to_string())
        .filter(|path| !path.is_empty())
}

fn metadata_value(metadata_arr: Option<&StringArray>, row: usize) -> Option<serde_json::Value> {
    let metadata_arr = metadata_arr.filter(|arr| !arr.is_null(row))?;
    serde_json::from_str(metadata_arr.value(row)).ok()
}

fn path_from_metadata(metadata: &serde_json::Value) -> Option<String> {
    metadata
        .get("file_path")
        .and_then(|value| value.as_str())
        .map(String::from)
}

fn hash_from_metadata(metadata: &serde_json::Value) -> Option<String> {
    metadata
        .get("file_hash")
        .and_then(|value| value.as_str())
        .map(String::from)
}

fn string_column<'a>(batch: &'a RecordBatch, column: &str) -> Option<&'a StringArray> {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
}
