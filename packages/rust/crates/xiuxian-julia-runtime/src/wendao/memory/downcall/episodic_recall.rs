//! Downcall helpers for Julia episodic-recall requests.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

use crate::wendao::memory::host::{
    EpisodicRecallQueryInputs, MemoryProjectionRow,
    build_episodic_recall_request_rows_from_projection,
};
use crate::wendao::memory::{
    MemoryJuliaEpisodicRecallScoreRow, fetch_memory_julia_episodic_recall_score_rows,
};

/// Compose Rust memory projection staging plus the Julia `episodic_recall`
/// downcall in one plugin-owned helper.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when host projection staging fails, the
/// Flight roundtrip fails, or the Julia response cannot be decoded.
pub async fn fetch_episodic_recall_score_rows_from_projection(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    query: &EpisodicRecallQueryInputs,
    projection_rows: &[MemoryProjectionRow],
) -> Result<Vec<MemoryJuliaEpisodicRecallScoreRow>, RepoIntelligenceError> {
    let request_rows = build_episodic_recall_request_rows_from_projection(query, projection_rows)?;
    fetch_memory_julia_episodic_recall_score_rows(runtime, request_rows.as_slice()).await
}
