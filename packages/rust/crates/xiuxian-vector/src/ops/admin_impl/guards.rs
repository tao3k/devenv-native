impl VectorStore {
    pub(crate) async fn invalidate_cached_table(&self, table_name: &str) {
        let mut cache = self.datasets.write().await;
        let _ = cache.remove(table_name);
    }

    pub(crate) async fn open_table_or_err(
        &self,
        table_name: &str,
    ) -> Result<Dataset, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Err(VectorStoreError::TableNotFound(table_name.to_string()));
        }
        {
            let mut cache = self.datasets.write().await;
            if let Some(cached) = cache.get(table_name) {
                return Ok(cached);
            }
        }
        let uri = table_path.to_string_lossy();
        let dataset = self.open_dataset_at_uri(uri.as_ref()).await?;
        {
            let mut cache = self.datasets.write().await;
            if let Some(cached) = cache.get(table_name) {
                return Ok(cached);
            }
            cache.insert(table_name.to_string(), dataset.clone());
        }
        Ok(dataset)
    }

    fn ensure_non_reserved_column(column: &str) -> Result<(), VectorStoreError> {
        if Self::is_reserved_column(column) {
            return Err(VectorStoreError::General(format!(
                "Column '{column}' is reserved and cannot be altered or dropped"
            )));
        }
        Ok(())
    }

    fn is_reserved_column(column: &str) -> bool {
        matches!(
            column,
            crate::ID_COLUMN
                | crate::VECTOR_COLUMN
                | crate::CONTENT_COLUMN
                | crate::METADATA_COLUMN
                | crate::THREAD_ID_COLUMN
                | crate::SKILL_NAME_COLUMN
                | crate::CATEGORY_COLUMN
                | crate::TOOL_NAME_COLUMN
                | crate::FILE_PATH_COLUMN
                | crate::ROUTING_KEYWORDS_COLUMN
                | crate::INTENTS_COLUMN
        )
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/ops/admin_impl/guards.rs"]
mod tests;
