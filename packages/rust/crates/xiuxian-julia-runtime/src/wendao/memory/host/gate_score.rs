//! Host-side staging for Julia memory-gate scoring request rows.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use crate::wendao::memory::{
    MemoryJuliaGateScoreRequestRow, build_memory_julia_gate_score_request_batch,
};

use super::staging::{optional_text, required_text, validate_probability};
use super::types::{MemoryLifecycleState, MemoryUtilityLedger};

const SURFACE: &str = "memory Julia memory_gate_score host staging";

/// Host-owned evidence row for one Julia `memory_gate_score` downcall.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGateScoreEvidenceRow {
    /// Stable host memory id used as the join key across recommendation rows.
    pub memory_id: String,
    /// Optional scenario pack forwarded into the Julia compute lane.
    pub scenario_pack: Option<String>,
    /// Rust-owned utility ledger for the target memory item.
    pub ledger: MemoryUtilityLedger,
    /// Current Rust-owned lifecycle state.
    pub current_state: MemoryLifecycleState,
}

/// Host-owned gate-score signals that are not stored directly on an episode.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGateScoreEvidenceSignals {
    /// `ReAct` revalidation score in `[0, 1]`.
    pub react_revalidation_score: f32,
    /// Graph consistency score in `[0, 1]`.
    pub graph_consistency_score: f32,
    /// Omega alignment score in `[0, 1]`.
    pub omega_alignment_score: f32,
    /// Current Rust-owned lifecycle state.
    pub current_state: MemoryLifecycleState,
}

/// Build typed Julia `memory_gate_score` request rows from Rust-owned gate
/// evidence.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any host evidence row violates the
/// staged `memory_gate_score` request contract.
pub fn build_memory_gate_score_request_rows_from_evidence(
    evidence_rows: &[MemoryGateScoreEvidenceRow],
) -> Result<Vec<MemoryJuliaGateScoreRequestRow>, RepoIntelligenceError> {
    evidence_rows.iter().map(build_request_row).collect()
}

/// Build one Julia `memory_gate_score` request batch from Rust-owned gate
/// evidence.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the evidence is empty or any staged
/// row violates the Julia `memory_gate_score` request contract.
pub fn build_memory_gate_score_request_batch_from_evidence(
    evidence_rows: &[MemoryGateScoreEvidenceRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let rows = build_memory_gate_score_request_rows_from_evidence(evidence_rows)?;
    if rows.is_empty() {
        return Err(staging_error(
            "memory Julia memory_gate_score host staging requires at least one evidence row",
        ));
    }
    build_memory_julia_gate_score_request_batch(&rows)
}

fn build_request_row(
    evidence_row: &MemoryGateScoreEvidenceRow,
) -> Result<MemoryJuliaGateScoreRequestRow, RepoIntelligenceError> {
    let memory_id = required_text(&evidence_row.memory_id, "memory_id", SURFACE)?;
    let scenario_pack = optional_text(evidence_row.scenario_pack.as_deref());
    validate_probability(
        "react_revalidation_score",
        evidence_row.ledger.react_revalidation_score,
        SURFACE,
    )?;
    validate_probability(
        "graph_consistency_score",
        evidence_row.ledger.graph_consistency_score,
        SURFACE,
    )?;
    validate_probability(
        "omega_alignment_score",
        evidence_row.ledger.omega_alignment_score,
        SURFACE,
    )?;
    validate_probability("q_value", evidence_row.ledger.q_value, SURFACE)?;
    validate_probability("failure_rate", evidence_row.ledger.failure_rate, SURFACE)?;
    validate_probability("ttl_score", evidence_row.ledger.ttl_score, SURFACE)?;

    Ok(MemoryJuliaGateScoreRequestRow {
        memory_id,
        scenario_pack,
        react_revalidation_score: evidence_row.ledger.react_revalidation_score,
        graph_consistency_score: evidence_row.ledger.graph_consistency_score,
        omega_alignment_score: evidence_row.ledger.omega_alignment_score,
        q_value: evidence_row.ledger.q_value,
        usage_count: evidence_row.ledger.usage_count,
        failure_rate: evidence_row.ledger.failure_rate,
        ttl_score: evidence_row.ledger.ttl_score,
        current_state: evidence_row.current_state.as_str().into(),
    })
}

fn staging_error(message: impl Into<String>) -> RepoIntelligenceError {
    super::staging::staging_error(SURFACE, message)
}
