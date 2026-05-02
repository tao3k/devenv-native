//! Arrow request batch construction for Plugin Arrow exchange scoring.

use std::{collections::BTreeMap, fmt, sync::Arc};

use arrow_array::{FixedSizeListArray, Float32Array, Float64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field};
use xiuxian_wendao_core::repo_intelligence::{RepoIntelligenceError, julia_arrow_request_schema};

use super::errors::contract_request_error;
use super::metadata::{attach_plugin_arrow_request_metadata, plugin_arrow_request_trace_id};

fn plugin_arrow_vector_item_field() -> Arc<Field> {
    Arc::new(Field::new("item", DataType::Float32, true))
}

/// One request row for the `WendaoArrow` `v1` plugin rerank contract.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginArrowRequestRow {
    /// Stable document identifier for the candidate row.
    pub doc_id: String,
    /// Coarse Rust-side retrieval score.
    pub vector_score: f64,
    /// Candidate embedding forwarded to the plugin transport.
    pub embedding: Vec<f32>,
}

/// One scored candidate that still needs its embedding resolved before one
/// `WendaoArrow` request batch can be materialized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PluginArrowScoredCandidate<'a> {
    /// Stable document identifier for the candidate row.
    pub doc_id: &'a str,
    /// Coarse Rust-side retrieval score.
    pub vector_score: f64,
}

/// One generic projection from owner-local doc-score pairs into the stable
/// plugin-arrow request-shaping surface.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginArrowCandidateProjection<'a> {
    /// Owned doc ids used for embedding lookup calls.
    pub doc_ids: Vec<String>,
    /// Borrowed scored candidates used by request-batch shaping.
    pub candidates: Vec<PluginArrowScoredCandidate<'a>>,
}

/// Error returned when runtime cannot shape one `WendaoArrow` request batch
/// from scored candidates and one embedding lookup table.
#[derive(Debug)]
pub enum PluginArrowRequestBatchBuildError {
    /// One scored candidate had no fetched embedding in the supplied lookup.
    MissingEmbedding {
        /// Candidate document id that was not present in the embedding lookup.
        doc_id: String,
    },
    /// Low-level `WendaoArrow` request-batch construction failed.
    Build(RepoIntelligenceError),
}

impl fmt::Display for PluginArrowRequestBatchBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

impl std::error::Error for PluginArrowRequestBatchBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingEmbedding { .. } => None,
            Self::Build(error) => Some(error),
        }
    }
}

impl From<RepoIntelligenceError> for PluginArrowRequestBatchBuildError {
    fn from(error: RepoIntelligenceError) -> Self {
        Self::Build(error)
    }
}

/// Project generic doc-score pairs into owned doc ids plus scored candidates.
#[must_use]
pub fn project_plugin_arrow_scored_candidates<'a, I>(rows: I) -> PluginArrowCandidateProjection<'a>
where
    I: IntoIterator<Item = (&'a str, f64)>,
{
    let mut doc_ids = Vec::new();
    let mut candidates = Vec::new();
    for (doc_id, vector_score) in rows {
        doc_ids.push(doc_id.to_string());
        candidates.push(PluginArrowScoredCandidate {
            doc_id,
            vector_score,
        });
    }
    PluginArrowCandidateProjection {
        doc_ids,
        candidates,
    }
}

/// Build one `WendaoArrow` `v1` plugin request batch from typed Rust rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request rows are empty, any row
/// carries an empty or mismatched embedding, or the query vector is empty.
pub fn build_plugin_arrow_request_batch(
    rows: &[PluginArrowRequestRow],
    query_vector: &[f32],
) -> Result<RecordBatch, RepoIntelligenceError> {
    if rows.is_empty() {
        return Err(contract_request_error(
            "WendaoArrow request batch requires at least one row",
        ));
    }
    if query_vector.is_empty() {
        return Err(contract_request_error(
            "WendaoArrow request batch requires a non-empty query vector",
        ));
    }

    let expected_dim = query_vector.len();
    let Some(vector_dim) = i32::try_from(expected_dim).ok() else {
        return Err(contract_request_error(format!(
            "query vector dimension {expected_dim} exceeds i32 range"
        )));
    };

    let mut doc_ids = Vec::with_capacity(rows.len());
    let mut vector_scores = Vec::with_capacity(rows.len());
    let mut embedding_values = Vec::with_capacity(rows.len() * expected_dim);
    let mut query_embedding_values = Vec::with_capacity(rows.len() * expected_dim);

    for row in rows {
        if row.doc_id.trim().is_empty() {
            return Err(contract_request_error(
                "WendaoArrow request row `doc_id` must be non-empty",
            ));
        }
        if row.embedding.len() != expected_dim {
            return Err(contract_request_error(format!(
                "embedding dimension mismatch for doc_id `{}`: expected {}, found {}",
                row.doc_id,
                expected_dim,
                row.embedding.len()
            )));
        }

        doc_ids.push(row.doc_id.as_str());
        vector_scores.push(row.vector_score);
        embedding_values.extend_from_slice(row.embedding.as_slice());
        query_embedding_values.extend_from_slice(query_vector);
    }

    let schema = julia_arrow_request_schema(vector_dim);

    let embedding = FixedSizeListArray::try_new(
        plugin_arrow_vector_item_field(),
        vector_dim,
        Arc::new(Float32Array::from(embedding_values)),
        None,
    )
    .map_err(|error| contract_request_error(error.to_string()))?;
    let query_embedding = FixedSizeListArray::try_new(
        plugin_arrow_vector_item_field(),
        vector_dim,
        Arc::new(Float32Array::from(query_embedding_values)),
        None,
    )
    .map_err(|error| contract_request_error(error.to_string()))?;

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(doc_ids)),
            Arc::new(Float64Array::from(vector_scores)),
            Arc::new(embedding),
            Arc::new(query_embedding),
        ],
    )
    .map_err(|error| contract_request_error(error.to_string()))
}

/// Build one `WendaoArrow` request batch from scored candidates plus one
/// embedding lookup table.
///
/// # Errors
///
/// Returns [`PluginArrowRequestBatchBuildError`] when one scored candidate has
/// no embedding in the provided lookup or the final request batch cannot be
/// materialized.
pub fn build_plugin_arrow_request_batch_from_embeddings(
    candidates: &[PluginArrowScoredCandidate<'_>],
    embeddings_by_doc_id: &BTreeMap<String, Vec<f32>>,
    query_vector: &[f32],
) -> Result<RecordBatch, PluginArrowRequestBatchBuildError> {
    let mut rows = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let Some(embedding) = embeddings_by_doc_id.get(candidate.doc_id).cloned() else {
            return Err(PluginArrowRequestBatchBuildError::MissingEmbedding {
                doc_id: candidate.doc_id.to_string(),
            });
        };
        rows.push(PluginArrowRequestRow {
            doc_id: candidate.doc_id.to_string(),
            vector_score: candidate.vector_score,
            embedding,
        });
    }
    build_plugin_arrow_request_batch(&rows, query_vector).map_err(Into::into)
}

/// Build one metadata-bearing `WendaoArrow` request batch from scored
/// candidates plus one embedding lookup table.
///
/// # Errors
///
/// Returns [`PluginArrowRequestBatchBuildError`] when one scored candidate has
/// no embedding in the provided lookup, the request batch cannot be
/// materialized, or request metadata cannot be attached.
pub fn build_plugin_arrow_request_batch_from_embeddings_with_metadata(
    candidates: &[PluginArrowScoredCandidate<'_>],
    embeddings_by_doc_id: &BTreeMap<String, Vec<f32>>,
    query_vector: &[f32],
    provider_id: &str,
    query_text: &str,
    schema_version: &str,
) -> Result<RecordBatch, PluginArrowRequestBatchBuildError> {
    let batch = build_plugin_arrow_request_batch_from_embeddings(
        candidates,
        embeddings_by_doc_id,
        query_vector,
    )?;
    attach_plugin_arrow_request_metadata(
        &batch,
        plugin_arrow_request_trace_id(provider_id, query_text).as_str(),
        schema_version,
    )
    .map_err(Into::into)
}
