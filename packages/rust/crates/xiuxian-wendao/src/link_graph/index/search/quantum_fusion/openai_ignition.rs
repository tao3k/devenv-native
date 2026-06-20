//! `link_graph::index::search::quantum_fusion::openai_ignition` owns Wendao search quantum fusion openai ignition behavior.

use super::scoring::distance_to_score;
use super::semantic_ignition::{QuantumSemanticIgnition, QuantumSemanticIgnitionFuture};
#[cfg(feature = "julia")]
use crate::analyzers::RepoIntelligenceError;
use crate::link_graph::models::{QuantumAnchorHit, QuantumSemanticSearchRequest};
#[cfg(feature = "julia")]
use arrow::record_batch::RecordBatch;
use std::sync::OnceLock;
use std::time::Duration;
use thiserror::Error;
use xiuxian_db_store::{SearchOptions, VectorStore, VectorStoreError};
#[cfg(feature = "julia")]
use xiuxian_wendao_runtime::transport::{
    PluginArrowVectorStoreRequestBatchInput, PluginArrowVectorStoreRequestBuildError,
    build_plugin_arrow_request_batch_from_vector_store,
    build_plugin_arrow_request_batch_from_vector_store_with_metadata,
};

/// Semantic ignition adapter backed by precomputed vectors by default.
///
/// Text-to-vector OpenAI-compatible embedding transport is available only when
/// the explicit `llm` feature is enabled. Default Wendao builds keep provider
/// execution outside this crate.
#[derive(Clone)]
pub struct OpenAiCompatibleSemanticIgnition {
    store: VectorStore,
    table_name: String,
    search_options: SearchOptions,
    backend_name: String,
    embedding_client: reqwest::Client,
    embedding_base_url: String,
    embedding_model: Option<String>,
}

impl OpenAiCompatibleSemanticIgnition {
    /// Create an OpenAI-compatible semantic ignition adapter.
    ///
    /// `embedding_base_url` is normalized by `xiuxian-llm` into
    /// `{base}/v1/embeddings` at request time.
    pub fn new(
        store: VectorStore,
        table_name: impl Into<String>,
        embedding_base_url: impl Into<String>,
    ) -> Self {
        Self {
            store,
            table_name: table_name.into(),
            search_options: SearchOptions::default(),
            backend_name: "openai-compatible+lance-vector-store".to_string(),
            embedding_client: shared_embedding_client().clone(),
            embedding_base_url: embedding_base_url.into(),
            embedding_model: None,
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

    /// Override the HTTP client used for embedding calls.
    ///
    /// This can be used to inject authentication headers for provider
    /// gateways that require API keys.
    #[must_use]
    pub fn with_embedding_client(mut self, client: reqwest::Client) -> Self {
        self.embedding_client = client;
        self
    }

    /// Set an explicit embedding model name for the OpenAI-compatible request.
    #[must_use]
    pub fn with_embedding_model(mut self, model: impl Into<String>) -> Self {
        self.embedding_model = Some(model.into());
        self
    }

    /// Override the base URL used for OpenAI-compatible embedding calls.
    #[must_use]
    pub fn with_embedding_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.embedding_base_url = base_url.into();
        self
    }

    #[cfg(feature = "julia")]
    async fn resolve_query_vector(
        &self,
        request: QuantumSemanticSearchRequest<'_>,
    ) -> Result<Vec<f32>, OpenAiCompatibleSemanticIgnitionError> {
        let query_vector = request.query_vector.to_vec();
        if !query_vector.is_empty() {
            return Ok(query_vector);
        }

        let query_text = request
            .query_text
            .filter(|value| !value.trim().is_empty())
            .ok_or(OpenAiCompatibleSemanticIgnitionError::MissingQuerySignal)?;
        self.embed_query_text(query_text).await
    }

    #[cfg(feature = "julia")]
    async fn embed_query_text(
        &self,
        _query_text: &str,
    ) -> Result<Vec<f32>, OpenAiCompatibleSemanticIgnitionError> {
        Err(OpenAiCompatibleSemanticIgnitionError::EmbeddingProviderDisabled)
    }

    #[cfg(feature = "julia")]
    async fn resolve_plugin_rerank_query_vector(
        &self,
        request: QuantumSemanticSearchRequest<'_>,
    ) -> Result<Vec<f32>, OpenAiCompatiblePluginRerankRequestError> {
        let query_vector = self
            .resolve_query_vector(request)
            .await
            .map_err(OpenAiCompatiblePluginRerankRequestError::Ignition)?;
        Ok(query_vector)
    }

    #[cfg(feature = "julia")]
    fn validate_plugin_rerank_anchors(
        anchors: &[QuantumAnchorHit],
    ) -> Result<(), OpenAiCompatiblePluginRerankRequestError> {
        if anchors.is_empty() {
            return Err(OpenAiCompatiblePluginRerankRequestError::Build(
                RepoIntelligenceError::AnalysisFailed {
                    message: "cannot build plugin rerank request from an empty anchor set"
                        .to_string(),
                },
            ));
        }
        Ok(())
    }

    /// Build a `WendaoArrow` `v1` plugin rerank request batch for one
    /// OpenAI-compatible semantic-ignition result set.
    ///
    /// # Errors
    ///
    /// Returns [`OpenAiCompatibleJuliaRequestError`] when the effective
    /// query vector cannot be resolved, candidate embeddings cannot be fetched,
    /// or the `WendaoArrow` request batch cannot be assembled.
    #[cfg(feature = "julia")]
    pub async fn build_plugin_rerank_request_batch(
        &self,
        request: QuantumSemanticSearchRequest<'_>,
        anchors: &[QuantumAnchorHit],
    ) -> Result<RecordBatch, OpenAiCompatiblePluginRerankRequestError> {
        Self::validate_plugin_rerank_anchors(anchors)?;
        let query_vector = self.resolve_plugin_rerank_query_vector(request).await?;
        build_plugin_arrow_request_batch_from_vector_store(
            &self.store,
            self.table_name.as_str(),
            anchors
                .iter()
                .map(|anchor| (anchor.anchor_id.clone(), anchor.vector_score)),
            &query_vector,
        )
        .await
        .map_err(map_openai_plugin_request_build_error)
    }

    /// Build a plugin rerank request batch and attach transport metadata for
    /// one OpenAI-compatible semantic-ignition result set.
    ///
    /// # Errors
    ///
    /// Returns [`OpenAiCompatiblePluginRerankRequestError`] when the effective
    /// query vector cannot be resolved or the base request batch cannot be
    /// assembled, and returns a string error when transport metadata cannot be
    /// attached to the batch.
    #[cfg(feature = "julia")]
    pub(crate) async fn build_plugin_rerank_request_batch_with_metadata(
        &self,
        request: QuantumSemanticSearchRequest<'_>,
        anchors: &[QuantumAnchorHit],
        provider_id: &str,
        query_text: &str,
        schema_version: &str,
    ) -> Result<RecordBatch, String> {
        Self::validate_plugin_rerank_anchors(anchors)
            .map_err(|error| format!("failed to build plugin rerank request batch: {error}"))?;
        let query_vector = self
            .resolve_plugin_rerank_query_vector(request)
            .await
            .map_err(|error| format!("failed to build plugin rerank request batch: {error}"))?;
        build_plugin_arrow_request_batch_from_vector_store_with_metadata(
            PluginArrowVectorStoreRequestBatchInput {
                store: &self.store,
                table_name: self.table_name.as_str(),
                rows: anchors
                    .iter()
                    .map(|anchor| (anchor.anchor_id.clone(), anchor.vector_score)),
                query_vector: &query_vector,
                provider_id,
                query_text,
                schema_version,
            },
        )
        .await
        .map_err(|error| format!("failed to build plugin rerank request batch: {error}"))
    }
}

fn shared_embedding_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .brotli(true)
            .deflate(true)
            .gzip(true)
            .zstd(true)
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

/// Error returned when OpenAI-compatible semantic ignition cannot produce
/// anchor hits.
#[derive(Debug, Error)]
pub enum OpenAiCompatibleSemanticIgnitionError {
    /// Request did not provide either a precomputed vector or query text.
    #[error("semantic request missing both query_vector and query_text")]
    MissingQuerySignal,
    /// OpenAI-compatible embedding request failed or returned invalid payload.
    #[error("openai-compatible embedding unavailable")]
    EmbeddingUnavailable,
    /// This build does not enable in-process embedding provider transport.
    #[error(
        "openai-compatible embedding provider transport is owned by the external LLM provider boundary"
    )]
    EmbeddingProviderDisabled,
    /// OpenAI-compatible embedding succeeded but returned an empty vector.
    #[error("openai-compatible embedding returned empty vector")]
    EmptyEmbeddingVector,
    /// Vector store search failed.
    #[error("vector store search failed: {0}")]
    VectorStore(#[from] VectorStoreError),
}

/// Error returned when OpenAI-compatible semantic ignition cannot assemble one
/// `WendaoArrow` plugin rerank request batch.
#[cfg(feature = "julia")]
#[derive(Debug, Error)]
pub enum OpenAiCompatiblePluginRerankRequestError {
    /// Effective query-vector resolution failed.
    #[error("failed to resolve query vector for plugin rerank request")]
    Ignition(#[source] OpenAiCompatibleSemanticIgnitionError),
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
fn map_openai_plugin_request_build_error(
    error: PluginArrowVectorStoreRequestBuildError,
) -> OpenAiCompatiblePluginRerankRequestError {
    match error {
        PluginArrowVectorStoreRequestBuildError::VectorStore(error) => {
            OpenAiCompatiblePluginRerankRequestError::VectorStore(error)
        }
        PluginArrowVectorStoreRequestBuildError::MissingEmbedding { doc_id } => {
            OpenAiCompatiblePluginRerankRequestError::MissingEmbedding { anchor_id: doc_id }
        }
        PluginArrowVectorStoreRequestBuildError::Build(error) => {
            OpenAiCompatiblePluginRerankRequestError::Build(error)
        }
    }
}

impl QuantumSemanticIgnition for OpenAiCompatibleSemanticIgnition {
    type Error = OpenAiCompatibleSemanticIgnitionError;

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
        let limit = request.candidate_limit.max(1);
        let embedding_client = self.embedding_client.clone();
        let embedding_base_url = self.embedding_base_url.clone();
        let embedding_model = self.embedding_model.clone();
        let query_text = request.query_text.map(str::to_string);
        let query_vector = request.query_vector.to_vec();

        Box::pin(async move {
            let effective_query_vector = if query_vector.is_empty() {
                let query_text = query_text
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or(OpenAiCompatibleSemanticIgnitionError::MissingQuerySignal)?;
                embed_query_text(
                    &embedding_client,
                    embedding_base_url.as_str(),
                    query_text,
                    embedding_model.as_deref(),
                )
                .await?
            } else {
                query_vector
            };

            let results = store
                .search_optimized(&table_name, effective_query_vector, limit, options)
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

async fn embed_query_text(
    _embedding_client: &reqwest::Client,
    _embedding_base_url: &str,
    _query_text: &str,
    _embedding_model: Option<&str>,
) -> Result<Vec<f32>, OpenAiCompatibleSemanticIgnitionError> {
    Err(OpenAiCompatibleSemanticIgnitionError::EmbeddingProviderDisabled)
}

#[cfg(all(test, feature = "julia"))]
#[path = "../../../../../tests/unit/link_graph/index/search/quantum_fusion/openai_ignition.rs"]
mod tests;
