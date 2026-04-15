use super::{
    HybridSearchResult, KEYWORD_WEIGHT, RRF_K, SEMANTIC_WEIGHT, SearchOptions, VectorStore,
    VectorStoreError, apply_weighted_rrf, f64_to_f32_saturating,
};

impl VectorStore {
    /// Unified keyword search entrypoint for the Lance FTS path.
    ///
    /// # Errors
    ///
    /// Returns an error if keyword search is disabled or FTS query execution
    /// fails after one best-effort index bootstrap attempt.
    pub async fn keyword_search(
        &self,
        table_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::skill::ToolSearchResult>, VectorStoreError> {
        if !self.keyword_search_enabled {
            return Err(VectorStoreError::General(
                "Keyword search is not enabled.".to_string(),
            ));
        }
        match self.search_fts(table_name, query, limit, None).await {
            Ok(results) => Ok(results),
            Err(error) => {
                log::debug!(
                    "keyword_search: Lance FTS query failed for `{table_name}`, retrying after index build: {error}"
                );
                self.create_fts_index(table_name).await?;
                self.search_fts(table_name, query, limit, None).await
            }
        }
    }

    /// Hybrid search combining vector similarity and keyword (`BM25`) search.
    /// Vector and keyword queries run in parallel via `try_join!` to reduce latency;
    /// vector failure fails fast, keyword failure falls back to empty.
    ///
    /// # Errors
    ///
    /// Returns an error if vector search fails.
    pub async fn hybrid_search(
        &self,
        table_name: &str,
        query: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<HybridSearchResult>, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Err(VectorStoreError::TableNotFound(table_name.to_string()));
        }

        let vector_fut = self.search_optimized(
            table_name,
            query_vector,
            limit * 2,
            SearchOptions::default(),
        );
        let kw_fut = async {
            match self.keyword_search(table_name, query, limit * 2).await {
                Ok(v) => Ok(v),
                Err(e) => {
                    log::debug!("Keyword search failed, falling back to vector-only: {e}");
                    Ok(Vec::new())
                }
            }
        };
        let (vector_results, kw_results) = tokio::try_join!(vector_fut, kw_fut)?;

        let vector_scores: Vec<(String, f32)> = vector_results
            .iter()
            .map(|r| (r.id.clone(), f64_to_f32_saturating(1.0 - r.distance)))
            .collect();

        let fused_results = apply_weighted_rrf(
            vector_scores,
            kw_results,
            RRF_K,
            SEMANTIC_WEIGHT,
            KEYWORD_WEIGHT,
            query,
        );

        Ok(fused_results.into_iter().take(limit).collect())
    }

    /// Legacy keyword indexing hook kept for compatibility.
    ///
    /// # Errors
    ///
    /// Returns an error when keyword search is not enabled for the store.
    pub fn index_keyword(
        &self,
        _name: &str,
        _description: &str,
        _category: &str,
        _keywords: &[String],
        _intents: &[String],
    ) -> Result<(), VectorStoreError> {
        self.ensure_keyword_index_hook_ready()?;
        Ok(())
    }

    /// Legacy bulk keyword indexing hook kept for compatibility.
    ///
    /// # Errors
    ///
    /// Returns an error when keyword search is not enabled for the store.
    pub fn bulk_index_keywords<I>(&self, docs: I) -> Result<(), VectorStoreError>
    where
        I: IntoIterator<Item = (String, String, String, Vec<String>, Vec<String>)>,
    {
        self.ensure_keyword_index_hook_ready()?;
        let _ = docs.into_iter().count();
        Ok(())
    }

    fn ensure_keyword_index_hook_ready(&self) -> Result<(), VectorStoreError> {
        if !self.keyword_search_enabled {
            return Err(VectorStoreError::General(
                "Keyword search is not enabled.".to_string(),
            ));
        }
        Ok(())
    }
}
