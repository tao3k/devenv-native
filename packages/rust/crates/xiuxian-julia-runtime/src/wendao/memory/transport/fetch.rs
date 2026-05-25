//! Repository-aware fetch helpers for Julia memory compute routes.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

use super::process::process_memory_julia_compute_flight_batches_for_runtime;
use crate::wendao::memory::{
    MemoryJuliaCalibrationArtifactRow, MemoryJuliaCalibrationRequestRow, MemoryJuliaComputeProfile,
    MemoryJuliaEpisodicRecallRequestRow, MemoryJuliaEpisodicRecallScoreRow,
    MemoryJuliaGateScoreRecommendationRow, MemoryJuliaGateScoreRequestRow,
    MemoryJuliaPlanTuningAdviceRow, MemoryJuliaPlanTuningRequestRow,
    build_memory_julia_calibration_request_batch, build_memory_julia_episodic_recall_request_batch,
    build_memory_julia_gate_score_request_batch, build_memory_julia_plan_tuning_request_batch,
    decode_memory_julia_calibration_artifact_rows, decode_memory_julia_episodic_recall_score_rows,
    decode_memory_julia_gate_score_recommendation_rows,
    decode_memory_julia_plan_tuning_advice_rows,
};

/// Send typed `episodic_recall` request rows to the configured memory-family
/// Julia provider and decode typed score rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when request materialization fails, the
/// Flight roundtrip fails, or the response cannot be decoded.
pub async fn fetch_memory_julia_episodic_recall_score_rows(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    rows: &[MemoryJuliaEpisodicRecallRequestRow],
) -> Result<Vec<MemoryJuliaEpisodicRecallScoreRow>, RepoIntelligenceError> {
    let batch = build_memory_julia_episodic_recall_request_batch(rows)?;
    let response_batches = process_memory_julia_compute_flight_batches_for_runtime(
        runtime,
        MemoryJuliaComputeProfile::EpisodicRecall,
        &[batch],
    )
    .await?;
    decode_memory_julia_episodic_recall_score_rows(response_batches.as_slice())
}

/// Send typed `memory_gate_score` request rows to the configured memory-family
/// Julia provider and decode typed recommendation rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when request materialization fails, the
/// Flight roundtrip fails, or the response cannot be decoded.
pub async fn fetch_memory_julia_gate_score_recommendation_rows(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    rows: &[MemoryJuliaGateScoreRequestRow],
) -> Result<Vec<MemoryJuliaGateScoreRecommendationRow>, RepoIntelligenceError> {
    let batch = build_memory_julia_gate_score_request_batch(rows)?;
    let response_batches = process_memory_julia_compute_flight_batches_for_runtime(
        runtime,
        MemoryJuliaComputeProfile::MemoryGateScore,
        &[batch],
    )
    .await?;
    decode_memory_julia_gate_score_recommendation_rows(response_batches.as_slice())
}

/// Send typed `memory_plan_tuning` request rows to the configured
/// memory-family Julia provider and decode typed advice rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when request materialization fails, the
/// Flight roundtrip fails, or the response cannot be decoded.
pub async fn fetch_memory_julia_plan_tuning_advice_rows(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    rows: &[MemoryJuliaPlanTuningRequestRow],
) -> Result<Vec<MemoryJuliaPlanTuningAdviceRow>, RepoIntelligenceError> {
    let batch = build_memory_julia_plan_tuning_request_batch(rows)?;
    let response_batches = process_memory_julia_compute_flight_batches_for_runtime(
        runtime,
        MemoryJuliaComputeProfile::MemoryPlanTuning,
        &[batch],
    )
    .await?;
    decode_memory_julia_plan_tuning_advice_rows(response_batches.as_slice())
}

/// Send typed `memory_calibration` request rows to the configured memory-family
/// Julia provider and decode typed artifact rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when request materialization fails, the
/// Flight roundtrip fails, or the response cannot be decoded.
pub async fn fetch_memory_julia_calibration_artifact_rows(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    rows: &[MemoryJuliaCalibrationRequestRow],
) -> Result<Vec<MemoryJuliaCalibrationArtifactRow>, RepoIntelligenceError> {
    let batch = build_memory_julia_calibration_request_batch(rows)?;
    let response_batches = process_memory_julia_compute_flight_batches_for_runtime(
        runtime,
        MemoryJuliaComputeProfile::MemoryCalibration,
        &[batch],
    )
    .await?;
    decode_memory_julia_calibration_artifact_rows(response_batches.as_slice())
}
