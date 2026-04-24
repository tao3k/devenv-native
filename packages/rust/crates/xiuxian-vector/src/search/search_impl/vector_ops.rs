use super::{
    CONTENT_COLUMN, ID_COLUMN, METADATA_COLUMN, SearchOptions, TryStreamExt, VECTOR_COLUMN, Value,
    VectorSearchResult, VectorStore, VectorStoreError, build_search_result_row,
    extract_vector_row_columns, json_to_lance_where, search_results_to_ipc,
};

impl VectorStore {
    /// Backward-compatible search wrapper.
    ///
    /// # Errors
    ///
    /// Returns an error when dataset/table access fails or the query cannot be executed.
    pub async fn search(
        &self,
        table_name: &str,
        query: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, VectorStoreError> {
        self.search_optimized(table_name, query, limit, SearchOptions::default())
            .await
    }

    /// Search with configurable scanner tuning for projection / read-ahead.
    ///
    /// # Errors
    ///
    /// Returns an error when the table does not exist, scanner setup fails, batch decoding
    /// fails, or Lance returns a query execution error.
    pub async fn search_optimized(
        &self,
        table_name: &str,
        query: Vec<f32>,
        limit: usize,
        options: SearchOptions,
    ) -> Result<Vec<VectorSearchResult>, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Err(VectorStoreError::TableNotFound(table_name.to_string()));
        }

        let dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let query_arr = lance::deps::arrow_array::Float32Array::from(query);
        let (pushdown_filter, metadata_filter) =
            Self::build_filter_plan(options.where_filter.as_deref());

        let mut scanner = dataset.scan();
        // When a filter is pushed down, Lance may use scalar indices (e.g. skill_name/category);
        // if filtering happens after ANN we request more candidates so enough pass the filter.
        let fetch_count = if pushdown_filter.is_some() {
            limit.saturating_mul(4).max(limit + 50)
        } else {
            limit.saturating_mul(2).max(limit + 10)
        };
        if !options.projected_columns.is_empty() {
            scanner.project(&options.projected_columns)?;
        }
        scanner.nearest(VECTOR_COLUMN, &query_arr, fetch_count)?;
        if let Some(batch_size) = options.batch_size {
            scanner.batch_size(batch_size);
        }
        if let Some(fragment_readahead) = options.fragment_readahead {
            scanner.fragment_readahead(fragment_readahead);
        }
        if let Some(batch_readahead) = options.batch_readahead {
            scanner.batch_readahead(batch_readahead);
        }
        if let Some(filter) = pushdown_filter {
            scanner.filter(&filter)?;
        }
        let scan_limit = options.scan_limit.unwrap_or(fetch_count);
        scanner.limit(Some(i64::try_from(scan_limit).unwrap_or(i64::MAX)), None)?;

        let mut stream = scanner.try_into_stream().await?;
        let mut results = Vec::with_capacity(limit.min(1024));

        while let Some(batch) = stream.try_next().await? {
            let columns = extract_vector_row_columns(&batch)?;
            for index in 0..batch.num_rows() {
                if let Some(result) =
                    build_search_result_row(index, &columns, metadata_filter.as_ref())
                {
                    results.push(result);
                }
            }
        }

        results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        Ok(results)
    }

    /// Search with configurable options; returns Arrow IPC stream bytes for zero-copy consumption in Python.
    /// See [search result batch contract](docs/reference/search-result-batch-contract.md).
    ///
    /// # Errors
    ///
    /// Returns an error if underlying vector search fails or IPC encoding fails.
    pub async fn search_optimized_ipc(
        &self,
        table_name: &str,
        query: Vec<f32>,
        limit: usize,
        options: SearchOptions,
    ) -> Result<Vec<u8>, VectorStoreError> {
        let results = self
            .search_optimized(table_name, query, limit, options.clone())
            .await?;
        search_results_to_ipc(&results, options.ipc_projection.as_deref())
            .map_err(VectorStoreError::General)
    }

    fn build_filter_plan(where_filter: Option<&str>) -> (Option<String>, Option<Value>) {
        let Some(filter) = where_filter.map(str::trim).filter(|f| !f.is_empty()) else {
            return (None, None);
        };

        if let Ok(json_filter) = serde_json::from_str::<Value>(filter) {
            let pushdown = if Self::is_pushdown_eligible_json(&json_filter) {
                let candidate = json_to_lance_where(&json_filter);
                if candidate.is_empty() {
                    None
                } else {
                    Some(candidate)
                }
            } else {
                None
            };
            return (pushdown, Some(json_filter));
        }

        (Some(filter.to_string()), None)
    }

    fn is_pushdown_eligible_json(expr: &Value) -> bool {
        let Some(obj) = expr.as_object() else {
            return false;
        };
        obj.keys().all(|key| {
            key == ID_COLUMN
                || key == CONTENT_COLUMN
                || key == METADATA_COLUMN
                || key == "_distance"
        })
    }
}
