use super::scoring::distance_to_score;
use super::semantic_ignition::{QuantumSemanticIgnition, QuantumSemanticIgnitionFuture};
#[cfg(feature = "julia")]
use crate::analyzers::RepoIntelligenceError;
use crate::link_graph::models::{QuantumAnchorHit, QuantumSemanticSearchRequest};
#[cfg(feature = "julia")]
use arrow::record_batch::RecordBatch;
#[cfg(feature = "julia")]
use thiserror::Error;
use xiuxian_db_store::{SearchOptions, VectorStore, VectorStoreError};
#[cfg(feature = "julia")]
use xiuxian_wendao_runtime::transport::{
    PluginArrowVectorStoreRequestBatchInput, PluginArrowVectorStoreRequestBuildError,
    build_plugin_arrow_request_batch_from_vector_store,
    build_plugin_arrow_request_batch_from_vector_store_with_metadata,
};

/// Semantic ignition adapter backed by the Lance vector-store facade.
#[derive(Clone)]
pub struct VectorStoreSemanticIgnition {
    store: VectorStore,
    table_name: String,
    search_options: SearchOptions,
    backend_name: String,
}

impl VectorStoreSemanticIgnition {
    /// Create a vector-backed ignition adapter for the given table.
    pub fn new(store: VectorStore, table_name: impl Into<String>) -> Self {
        Self {
            store,
            table_name: table_name.into(),
            search_options: SearchOptions::default(),
            backend_name: "lance-vector-store".to_string(),
        }
    }

    /// Override the search options passed to the vector store.
    #[must_use]
    pub fn with_search_options(mut self, options: SearchOptions) -> Self {
        self.search_options = options;
        self
    }

    /// Override the backend name surfaced in telemetry.
    #[must_use]
    pub fn with_backend_name(mut self, backend_name: impl Into<String>) -> Self {
        self.backend_name = backend_name.into();
        self
    }

    #[cfg(feature = "julia")]
    fn validate_plugin_rerank_candidates(
        anchors: &[QuantumAnchorHit],
    ) -> Result<(), VectorStorePluginRerankRequestError> {
        if anchors.is_empty() {
            return Err(VectorStorePluginRerankRequestError::Build(
                RepoIntelligenceError::AnalysisFailed {
                    message: "cannot build plugin rerank request from an empty anchor set"
                        .to_string(),
                },
            ));
        }
        Ok(())
    }

    /// Build a `WendaoArrow` `v1` plugin rerank request batch for the provided
    /// anchors.
    ///
    /// The request reuses `anchor_id` as the stable `doc_id` field because the
    /// quantum-fusion candidate identity is anchor-granular, not document-granular.
    ///
    /// # Errors
    ///
    /// Returns [`VectorStoreJuliaRequestError`] when candidate embeddings
    /// cannot be fetched from the vector store or the `WendaoArrow` request
    /// batch cannot be assembled.
    #[cfg(feature = "julia")]
    pub async fn build_plugin_rerank_request_batch(
        &self,
        request: QuantumSemanticSearchRequest<'_>,
        anchors: &[QuantumAnchorHit],
    ) -> Result<RecordBatch, VectorStorePluginRerankRequestError> {
        Self::validate_plugin_rerank_candidates(anchors)?;
        build_plugin_arrow_request_batch_from_vector_store(
            &self.store,
            self.table_name.as_str(),
            anchors
                .iter()
                .map(|anchor| (anchor.anchor_id.clone(), anchor.vector_score)),
            request.query_vector,
        )
        .await
        .map_err(map_vector_store_plugin_request_build_error)
    }

    /// Build a plugin rerank request batch and attach transport metadata for
    /// the provided anchors.
    ///
    /// # Errors
    ///
    /// Returns a string error when the base request batch cannot be assembled
    /// or transport metadata cannot be attached to the batch.
    #[cfg(feature = "julia")]
    pub(crate) async fn build_plugin_rerank_request_batch_with_metadata(
        &self,
        request: QuantumSemanticSearchRequest<'_>,
        anchors: &[QuantumAnchorHit],
        provider_id: &str,
        query_text: &str,
        schema_version: &str,
    ) -> Result<RecordBatch, String> {
        Self::validate_plugin_rerank_candidates(anchors)
            .map_err(|error| format!("failed to build plugin rerank request batch: {error}"))?;
        build_plugin_arrow_request_batch_from_vector_store_with_metadata(
            PluginArrowVectorStoreRequestBatchInput {
                store: &self.store,
                table_name: self.table_name.as_str(),
                rows: anchors
                    .iter()
                    .map(|anchor| (anchor.anchor_id.clone(), anchor.vector_score)),
                query_vector: request.query_vector,
                provider_id,
                query_text,
                schema_version,
            },
        )
        .await
        .map_err(|error| format!("failed to build plugin rerank request batch: {error}"))
    }
}

/// Error returned when the vector-backed semantic ignition cannot assemble one
/// `WendaoArrow` plugin rerank request batch.
#[cfg(feature = "julia")]
#[derive(Debug, Error)]
pub enum VectorStorePluginRerankRequestError {
    /// Fetching candidate embeddings from the vector store failed.
    #[error("failed to fetch candidate embeddings for plugin rerank request")]
    VectorStore(#[source] VectorStoreError),
    /// One anchor id from the semantic search result set had no stored vector.
    #[error("missing embedding for plugin rerank anchor `{anchor_id}`")]
    MissingEmbedding {
        /// Anchor id that could not be resolved into a stored embedding row.
        anchor_id: String,
    },
    /// `WendaoArrow` request batch construction failed.
    #[error("failed to build plugin rerank request batch")]
    Build(#[source] RepoIntelligenceError),
}

#[cfg(feature = "julia")]
fn map_vector_store_plugin_request_build_error(
    error: PluginArrowVectorStoreRequestBuildError,
) -> VectorStorePluginRerankRequestError {
    match error {
        PluginArrowVectorStoreRequestBuildError::VectorStore(error) => {
            VectorStorePluginRerankRequestError::VectorStore(error)
        }
        PluginArrowVectorStoreRequestBuildError::MissingEmbedding { doc_id } => {
            VectorStorePluginRerankRequestError::MissingEmbedding { anchor_id: doc_id }
        }
        PluginArrowVectorStoreRequestBuildError::Build(error) => {
            VectorStorePluginRerankRequestError::Build(error)
        }
    }
}

impl QuantumSemanticIgnition for VectorStoreSemanticIgnition {
    type Error = VectorStoreError;

    fn backend_name(&self) -> &str {
        self.backend_name.as_str()
    }

    fn search_anchors<'a>(
        &'a self,
        request: QuantumSemanticSearchRequest<'a>,
    ) -> QuantumSemanticIgnitionFuture<'a, Self::Error> {
        let store = self.store.clone();
        let table_name = self.table_name.clone();
        let options = self.search_options.clone();
        let query_vector = request.query_vector.to_vec();
        let limit = request.candidate_limit.max(1);

        Box::pin(async move {
            if query_vector.is_empty() || limit == 0 {
                return Ok(Vec::new());
            }
            let results = store
                .search_optimized(&table_name, query_vector, limit, options)
                .await?;
            Ok(results
                .into_iter()
                .map(|result| QuantumAnchorHit {
                    anchor_id: result.id,
                    vector_score: distance_to_score(result.distance),
                })
                .collect())
        })
    }
}

#[cfg(all(test, feature = "julia"))]
#[path = "../../../../../tests/unit/link_graph/index/search/quantum_fusion/vector_ignition.rs"]
mod tests;
