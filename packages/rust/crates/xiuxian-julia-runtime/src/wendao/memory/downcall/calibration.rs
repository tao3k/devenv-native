//! Downcall helpers for Julia memory calibration requests.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

use crate::wendao::memory::host::{
    MemoryCalibrationInputs, build_memory_calibration_request_rows_from_inputs,
};
use crate::wendao::memory::{
    MemoryJuliaCalibrationArtifactRow, fetch_memory_julia_calibration_artifact_rows,
};

/// Compose Rust calibration-input staging plus the Julia `memory_calibration`
/// downcall in one plugin-owned helper.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when host input staging fails, the Flight
/// roundtrip fails, or the Julia response cannot be decoded.
pub async fn fetch_calibration_artifact_rows_from_inputs(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    inputs: &[MemoryCalibrationInputs],
) -> Result<Vec<MemoryJuliaCalibrationArtifactRow>, RepoIntelligenceError> {
    let request_rows = build_memory_calibration_request_rows_from_inputs(inputs)?;
    fetch_memory_julia_calibration_artifact_rows(runtime, request_rows.as_slice()).await
}
