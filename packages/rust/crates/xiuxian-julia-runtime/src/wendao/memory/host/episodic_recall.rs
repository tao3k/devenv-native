//! Host-side staging for Julia episodic-recall request rows.

use arrow::record_batch::RecordBatch;
use xiuxian_memory_engine::MemoryProjectionRow;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use crate::wendao::memory::{
    MemoryJuliaEpisodicRecallRequestRow, build_memory_julia_episodic_recall_request_batch,
};

use super::staging::{
    optional_text, required_text, validate_embedding, validate_finite, validate_non_negative_finite,
};

const SURFACE: &str = "memory Julia episodic_recall host staging";

/// Host-owned shared query inputs for one Julia `episodic_recall` downcall.
#[derive(Debug, Clone, PartialEq)]
pub struct EpisodicRecallQueryInputs {
    /// Host query id used as the join key across candidate rows.
    pub query_id: String,
    /// Optional scenario pack forwarded into the Julia compute lane.
    pub scenario_pack: Option<String>,
    /// Optional raw query text.
    pub query_text: Option<String>,
    /// Semantic embedding for the current query.
    pub query_embedding: Vec<f32>,
    /// Semantic recall tuning weight.
    pub k1: f32,
    /// Utility rerank tuning weight.
    pub k2: f32,
    /// Fusion lambda.
    pub lambda: f32,
    /// Minimum accepted score.
    pub min_score: f32,
}

/// Build typed Julia `episodic_recall` request rows from a Rust memory
/// projection snapshot.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the shared query inputs or any
/// projection row violate the staged `episodic_recall` contract.
pub fn build_episodic_recall_request_rows_from_projection(
    query: &EpisodicRecallQueryInputs,
    projection_rows: &[MemoryProjectionRow],
) -> Result<Vec<MemoryJuliaEpisodicRecallRequestRow>, RepoIntelligenceError> {
    let normalized_query = normalize_query_inputs(query)?;
    projection_rows
        .iter()
        .map(|row| build_request_row(&normalized_query, row))
        .collect()
}

/// Build one Julia `episodic_recall` request batch from a Rust memory
/// projection snapshot.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the projection is empty or any
/// staged row violates the Julia `episodic_recall` request contract.
pub fn build_episodic_recall_request_batch_from_projection(
    query: &EpisodicRecallQueryInputs,
    projection_rows: &[MemoryProjectionRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let rows = build_episodic_recall_request_rows_from_projection(query, projection_rows)?;
    if rows.is_empty() {
        return Err(staging_error(
            "memory Julia episodic_recall host staging requires at least one projection row",
        ));
    }
    build_memory_julia_episodic_recall_request_batch(&rows)
}

#[derive(Debug, Clone, PartialEq)]
struct NormalizedEpisodicRecallQuery {
    query_id: String,
    scenario_pack: Option<String>,
    query_text: Option<String>,
    query_embedding: Vec<f32>,
    k1: f32,
    k2: f32,
    lambda: f32,
    min_score: f32,
}

fn normalize_query_inputs(
    query: &EpisodicRecallQueryInputs,
) -> Result<NormalizedEpisodicRecallQuery, RepoIntelligenceError> {
    let query_id = required_text(&query.query_id, "query_id", SURFACE)?;
    let scenario_pack = optional_text(query.scenario_pack.as_deref());
    let query_text = optional_text(query.query_text.as_deref());
    validate_embedding("query_embedding", &query.query_embedding, SURFACE)?;
    validate_non_negative_finite("k1", query.k1, SURFACE)?;
    validate_non_negative_finite("k2", query.k2, SURFACE)?;
    validate_non_negative_finite("lambda", query.lambda, SURFACE)?;
    validate_non_negative_finite("min_score", query.min_score, SURFACE)?;

    Ok(NormalizedEpisodicRecallQuery {
        query_id,
        scenario_pack,
        query_text,
        query_embedding: query.query_embedding.clone(),
        k1: query.k1,
        k2: query.k2,
        lambda: query.lambda,
        min_score: query.min_score,
    })
}

fn build_request_row(
    query: &NormalizedEpisodicRecallQuery,
    projection_row: &MemoryProjectionRow,
) -> Result<MemoryJuliaEpisodicRecallRequestRow, RepoIntelligenceError> {
    validate_projection_row(projection_row)?;

    Ok(MemoryJuliaEpisodicRecallRequestRow {
        query_id: query.query_id.clone().into(),
        scenario_pack: query.scenario_pack.clone(),
        scope: projection_row.scope.clone(),
        query_text: query.query_text.clone(),
        query_embedding: query.query_embedding.clone(),
        candidate_id: projection_row.episode_id.as_str().into(),
        intent_embedding: projection_row.intent_embedding.clone(),
        q_value: projection_row.q_value,
        success_count: projection_row.success_count,
        failure_count: projection_row.failure_count,
        retrieval_count: projection_row.retrieval_count,
        created_at_ms: projection_row.created_at_ms.get().into(),
        updated_at_ms: projection_row.updated_at_ms.get().into(),
        k1: query.k1,
        k2: query.k2,
        lambda: query.lambda,
        min_score: query.min_score,
    })
}

fn validate_projection_row(row: &MemoryProjectionRow) -> Result<(), RepoIntelligenceError> {
    required_text(&row.episode_id, "candidate_id", SURFACE)?;
    required_text(&row.scope, "scope", SURFACE)?;
    validate_embedding("intent_embedding", &row.intent_embedding, SURFACE)?;
    validate_finite("q_value", row.q_value, SURFACE)?;
    if row.updated_at_ms < row.created_at_ms {
        return Err(staging_error(format!(
            "projection row `{}` has updated_at_ms earlier than created_at_ms",
            row.episode_id.as_str()
        )));
    }
    Ok(())
}

fn staging_error(message: impl Into<String>) -> RepoIntelligenceError {
    super::staging::staging_error(SURFACE, message)
}
