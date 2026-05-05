use super::{
    Arc, Dataset, FragmentInfo, ID_COLUMN, METADATA_COLUMN, Result, TableColumnAlteration,
    TableColumnType, TableInfo, TableNewColumn, TableVersionInfo, TryStreamExt, VectorStore,
    VectorStoreError, is_dataset_not_found_or_invalid,
};

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
        let mut hash_map = std::collections::HashMap::new();
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
                    let meta = meta_arr.and_then(|ma| {
                        if ma.is_null(i) {
                            None
                        } else {
                            serde_json::from_str::<serde_json::Value>(ma.value(i)).ok()
                        }
                    });
                    let path = path_from_col.or_else(|| {
                        meta.as_ref().and_then(|m| {
                            m.get("file_path")
                                .and_then(|v| v.as_str())
                                .map(String::from)
                        })
                    });
                    let hash = meta.and_then(|m| {
                        m.get("file_hash")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                    });
                    if let Some(path) = path {
                        hash_map.insert(
                            path,
                            serde_json::json!({ "hash": hash.unwrap_or_default(), "id": id }),
                        );
                    }
                }
            }
        }
        serde_json::to_string(&hash_map).map_err(|e| VectorStoreError::General(e.to_string()))
    }
}
