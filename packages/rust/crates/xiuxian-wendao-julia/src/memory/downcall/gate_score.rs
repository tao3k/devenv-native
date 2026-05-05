//! Downcall helpers for Julia memory-gate scoring requests.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

use crate::memory::host::{
    MemoryGateScoreEvidenceRow, build_memory_gate_score_request_rows_from_evidence,
};
use crate::memory::{
    MemoryJuliaGateScoreRecommendationRow, fetch_memory_julia_gate_score_recommendation_rows,
};

/// Compose Rust gate-evidence staging plus the Julia `memory_gate_score`
/// downcall in one plugin-owned helper.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when host evidence staging fails, the
/// Flight roundtrip fails, or the Julia response cannot be decoded.
pub async fn fetch_gate_score_recommendation_rows_from_evidence(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    evidence_rows: &[MemoryGateScoreEvidenceRow],
) -> Result<Vec<MemoryJuliaGateScoreRecommendationRow>, RepoIntelligenceError> {
    let request_rows = build_memory_gate_score_request_rows_from_evidence(evidence_rows)?;
    fetch_memory_julia_gate_score_recommendation_rows(runtime, request_rows.as_slice()).await
}

#[cfg(test)]
#[path = "../../../tests/unit/memory/downcall/gate_score.rs"]
mod tests;
