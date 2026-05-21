//! `memory::julia::client` owns Wendao memory julia client behavior.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_julia::memory::downcall::{
    fetch_calibration_artifact_rows_from_inputs as fetch_calibration_artifact_rows_with_runtime,
    fetch_episodic_recall_score_rows_from_projection as fetch_episodic_recall_score_rows_with_runtime,
    fetch_gate_score_recommendation_rows_from_evidence as fetch_gate_score_recommendation_rows_with_runtime,
    fetch_plan_tuning_advice_rows_from_inputs as fetch_plan_tuning_advice_rows_with_runtime,
};
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

use super::runtime::{
    ensure_enabled_memory_julia_compute_runtime, resolve_enabled_memory_julia_compute_runtime,
};
use super::{
    EpisodicRecallQueryInputs, MemoryCalibrationInputs, MemoryGateScoreEvidenceRow,
    MemoryJuliaCalibrationArtifactRow, MemoryJuliaEpisodicRecallScoreRow,
    MemoryJuliaGateScoreRecommendationRow, MemoryJuliaPlanTuningAdviceRow, MemoryPlanTuningInputs,
    MemoryProjectionRow,
};

/// Configured host-facing client for memory-family Julia compute.
#[derive(Clone, Debug)]
pub struct ComputeClient {
    runtime: MemoryJuliaComputeRuntimeConfig,
}

impl ComputeClient {
    /// Build one host-facing client from already resolved runtime config.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when the provided runtime is disabled.
    pub fn new(runtime: MemoryJuliaComputeRuntimeConfig) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            runtime: ensure_enabled_memory_julia_compute_runtime(runtime, "memory-family client")?,
        })
    }

    /// Resolve merged Wendao settings and build one configured host-facing
    /// client for memory-family Julia compute.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when the configured runtime is
    /// disabled.
    pub fn configured() -> Result<Self, RepoIntelligenceError> {
        Self::new(resolve_enabled_memory_julia_compute_runtime(
            "memory-family client",
        )?)
    }

    /// Borrow the configured runtime backing this client.
    #[must_use]
    pub fn runtime(&self) -> &MemoryJuliaComputeRuntimeConfig {
        &self.runtime
    }

    /// Fetch Julia `episodic_recall` score rows through the configured host
    /// client.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when host projection staging fails,
    /// the Flight roundtrip fails, or the Julia response cannot be decoded.
    pub async fn fetch_episodic_recall_score_rows_from_projection(
        &self,
        query: &EpisodicRecallQueryInputs,
        projection_rows: &[MemoryProjectionRow],
    ) -> Result<Vec<MemoryJuliaEpisodicRecallScoreRow>, RepoIntelligenceError> {
        fetch_episodic_recall_score_rows_with_runtime(&self.runtime, query, projection_rows).await
    }

    /// Fetch Julia `memory_gate_score` recommendation rows through the
    /// configured host client.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when gate-evidence staging fails, the
    /// Flight roundtrip fails, or the Julia response cannot be decoded.
    pub async fn fetch_gate_score_recommendation_rows_from_evidence(
        &self,
        evidence_rows: &[MemoryGateScoreEvidenceRow],
    ) -> Result<Vec<MemoryJuliaGateScoreRecommendationRow>, RepoIntelligenceError> {
        fetch_gate_score_recommendation_rows_with_runtime(&self.runtime, evidence_rows).await
    }

    /// Fetch Julia `memory_plan_tuning` advice rows through the configured host
    /// client.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when tuning-input staging fails, the
    /// Flight roundtrip fails, or the Julia response cannot be decoded.
    pub async fn fetch_plan_tuning_advice_rows_from_inputs(
        &self,
        inputs: &[MemoryPlanTuningInputs],
    ) -> Result<Vec<MemoryJuliaPlanTuningAdviceRow>, RepoIntelligenceError> {
        fetch_plan_tuning_advice_rows_with_runtime(&self.runtime, inputs).await
    }

    /// Fetch Julia `memory_calibration` artifact rows through the configured
    /// host client.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when calibration-input staging fails,
    /// the Flight roundtrip fails, or the Julia response cannot be decoded.
    pub async fn fetch_calibration_artifact_rows_from_inputs(
        &self,
        inputs: &[MemoryCalibrationInputs],
    ) -> Result<Vec<MemoryJuliaCalibrationArtifactRow>, RepoIntelligenceError> {
        fetch_calibration_artifact_rows_with_runtime(&self.runtime, inputs).await
    }
}
