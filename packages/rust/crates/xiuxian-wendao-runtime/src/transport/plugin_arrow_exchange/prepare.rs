//! Request-row preparation helpers for Plugin Arrow exchange scoring.

use std::fmt;

use arrow_array::RecordBatch;
use xiuxian_db_store::{VectorStore, VectorStoreError};
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::PluginArrowProviderIdRef;
use super::metadata::{attach_plugin_arrow_request_metadata, plugin_arrow_request_trace_id};
use super::request::{PluginArrowRequestRow, build_plugin_arrow_request_batch};

/// Inputs used to build a metadata-bearing request batch from vector-store rows.
#[derive(Debug, Clone, Copy)]
pub struct PluginArrowVectorStoreRequestBatchInput<'a, I> {
    /// Vector store used to fetch candidate embeddings.
    pub store: &'a VectorStore,
    /// Vector-store table name.
    pub table_name: &'a str,
    /// Doc-score rows selected by the caller.
    pub rows: I,
    /// Query embedding forwarded to every request row.
    pub query_vector: &'a [f32],
    /// Provider id used to build the trace id.
    pub provider_id: PluginArrowProviderIdRef<'a>,
    /// User query text used to build the trace id.
    pub query_text: &'a str,
    /// Physical schema version attached to request metadata.
    pub schema_version: &'a str,
}

/// Error returned when runtime cannot fetch candidate embeddings from the
/// vector store and materialize one `WendaoArrow` request row set or batch.
#[derive(Debug)]
pub enum PluginArrowVectorStoreRequestBuildError {
    /// Fetching candidate embeddings from the vector store failed.
    VectorStore(VectorStoreError),
    /// One doc id from the candidate set had no stored embedding row.
    MissingEmbedding {
        /// Candidate document id that could not be resolved into a stored
        /// embedding row.
        doc_id: String,
    },
    /// Request row or request batch shaping failed.
    Build(RepoIntelligenceError),
}

impl fmt::Display for PluginArrowVectorStoreRequestBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VectorStore(_) => write!(
                formatter,
                "failed to fetch candidate embeddings for plugin rerank request"
            ),
            Self::MissingEmbedding { doc_id } => {
                write!(
                    formatter,
                    "missing embedding for plugin rerank candidate `{doc_id}`"
                )
            }
            Self::Build(_) => write!(formatter, "failed to build plugin rerank request batch"),
        }
    }
}

impl std::error::Error for PluginArrowVectorStoreRequestBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::VectorStore(error) => Some(error),
            Self::MissingEmbedding { .. } => None,
            Self::Build(error) => Some(error),
        }
    }
}

/// Build owned `WendaoArrow` request rows from doc-score pairs backed by one
/// vector-store embedding lookup.
///
/// # Errors
///
/// Returns [`PluginArrowVectorStoreRequestBuildError`] when the vector store
/// cannot fetch the required embeddings or one doc id from the supplied
/// doc-score pairs has no stored embedding row.
pub async fn prepare_plugin_arrow_request_rows_from_vector_store<I>(
    store: &VectorStore,
    table_name: &str,
    rows: I,
) -> Result<Vec<PluginArrowRequestRow>, PluginArrowVectorStoreRequestBuildError>
where
    I: IntoIterator<Item = (String, f64)>,
{
    let candidates = rows.into_iter().collect::<Vec<_>>();
    let doc_ids = candidates
        .iter()
        .map(|(doc_id, _)| doc_id.clone())
        .collect::<Vec<_>>();
    let embeddings_by_doc_id = store
        .fetch_embeddings_by_ids(table_name, &doc_ids)
        .await
        .map_err(PluginArrowVectorStoreRequestBuildError::VectorStore)?;

    let mut request_rows = Vec::with_capacity(candidates.len());
    for (doc_id, vector_score) in candidates {
        let Some(embedding) = embeddings_by_doc_id.get(doc_id.as_str()).cloned() else {
            return Err(PluginArrowVectorStoreRequestBuildError::MissingEmbedding { doc_id });
        };
        request_rows.push(PluginArrowRequestRow {
            doc_id,
            vector_score,
            embedding,
        });
    }
    Ok(request_rows)
}

/// Build one `WendaoArrow` request batch from doc-score pairs backed by one
/// vector-store embedding lookup.
///
/// # Errors
///
/// Returns [`PluginArrowVectorStoreRequestBuildError`] when the vector store
/// cannot fetch the required embeddings, one doc id is missing an embedding,
/// or the final request batch cannot be materialized.
pub async fn build_plugin_arrow_request_batch_from_vector_store<I>(
    store: &VectorStore,
    table_name: &str,
    rows: I,
    query_vector: &[f32],
) -> Result<RecordBatch, PluginArrowVectorStoreRequestBuildError>
where
    I: IntoIterator<Item = (String, f64)>,
{
    let request_rows =
        prepare_plugin_arrow_request_rows_from_vector_store(store, table_name, rows).await?;
    build_plugin_arrow_request_batch(&request_rows, query_vector)
        .map_err(PluginArrowVectorStoreRequestBuildError::Build)
}

/// Build one metadata-bearing `WendaoArrow` request batch from doc-score pairs
/// backed by one vector-store embedding lookup.
///
/// # Errors
///
/// Returns [`PluginArrowVectorStoreRequestBuildError`] when the vector store
/// cannot fetch the required embeddings, one doc id is missing an embedding,
/// the request batch cannot be materialized, or metadata attachment fails.
pub async fn build_plugin_arrow_request_batch_from_vector_store_with_metadata<I>(
    input: PluginArrowVectorStoreRequestBatchInput<'_, I>,
) -> Result<RecordBatch, PluginArrowVectorStoreRequestBuildError>
where
    I: IntoIterator<Item = (String, f64)>,
{
    let batch = build_plugin_arrow_request_batch_from_vector_store(
        input.store,
        input.table_name,
        input.rows,
        input.query_vector,
    )
    .await?;
    attach_plugin_arrow_request_metadata(
        &batch,
        plugin_arrow_request_trace_id(input.provider_id, input.query_text).as_str(),
        input.schema_version,
    )
    .map_err(PluginArrowVectorStoreRequestBuildError::Build)
}
