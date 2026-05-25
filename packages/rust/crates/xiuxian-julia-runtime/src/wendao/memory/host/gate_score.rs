//! Host-side staging for Julia memory-gate scoring request rows.

use arrow::record_batch::RecordBatch;
use xiuxian_memory_engine::{Episode, EpisodeStore, MemoryLifecycleState, MemoryUtilityLedger};
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use crate::wendao::memory::{
    MemoryJuliaGateScoreRequestRow, build_memory_julia_gate_score_request_batch,
};

use super::staging::{optional_text, required_text, validate_probability};

const SURFACE: &str = "memory Julia memory_gate_score host staging";

/// Borrowed host memory identifier used to look up a scored episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryGateScoreMemoryId<'a>(&'a str);

impl<'a> MemoryGateScoreMemoryId<'a> {
    /// Return the raw host memory identifier for store lookup.
    #[must_use]
    pub fn as_str(self) -> &'a str {
        self.0
    }
}

impl<'a> From<&'a str> for MemoryGateScoreMemoryId<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

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

/// Named inputs for looking up one gate-score evidence row from a store.
pub struct MemoryGateScoreStoreEvidenceInput<'a> {
    /// Episode store that owns the memory item.
    pub store: &'a EpisodeStore,
    /// Host memory identifier used for store lookup.
    pub memory_id: MemoryGateScoreMemoryId<'a>,
    /// Optional scenario pack forwarded into Julia.
    pub scenario_pack: Option<String>,
    /// Host-computed evidence signals for this memory item.
    pub signals: MemoryGateScoreEvidenceSignals,
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

/// Build one canonical gate-score evidence row from a host episode plus
/// already-computed evidence signals.
#[must_use]
pub fn build_memory_gate_score_evidence_row_from_episode(
    episode: &Episode,
    scenario_pack: Option<String>,
    signals: &MemoryGateScoreEvidenceSignals,
) -> MemoryGateScoreEvidenceRow {
    MemoryGateScoreEvidenceRow {
        memory_id: episode.id.clone(),
        scenario_pack,
        ledger: MemoryUtilityLedger::from_episode(
            episode,
            signals.react_revalidation_score,
            signals.graph_consistency_score,
            signals.omega_alignment_score,
        ),
        current_state: signals.current_state,
    }
}

/// Build one canonical gate-score evidence row from a stored episode id.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the requested episode does not
/// exist in the store.
pub fn build_memory_gate_score_evidence_row_from_store(
    input: MemoryGateScoreStoreEvidenceInput<'_>,
) -> Result<MemoryGateScoreEvidenceRow, RepoIntelligenceError> {
    let memory_id = input.memory_id.as_str();
    let Some(episode) = input.store.get(memory_id) else {
        return Err(staging_error(format!(
            "memory Julia memory_gate_score host staging could not find episode `{}`",
            memory_id.trim()
        )));
    };

    Ok(build_memory_gate_score_evidence_row_from_episode(
        &episode,
        input.scenario_pack,
        &input.signals,
    ))
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
