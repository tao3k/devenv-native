//! Host-side staging for Julia memory calibration request rows.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use crate::wendao::memory::{
    MemoryJuliaCalibrationRequestRow, build_memory_julia_calibration_request_batch,
};

use super::staging::{optional_text, required_text};

const SURFACE: &str = "memory Julia memory_calibration host staging";

/// Host-owned calibration job inputs for one Julia `memory_calibration`
/// downcall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryCalibrationInputs {
    /// Stable host-generated calibration job id.
    pub calibration_job_id: String,
    /// Optional scenario pack forwarded into the Julia compute lane.
    pub scenario_pack: Option<String>,
    /// Stable dataset reference for the calibration job.
    pub dataset_ref: String,
    /// Optimization objective label.
    pub objective: String,
    /// Serialized hyperparameter config payload.
    pub hyperparam_config: String,
}

/// Build typed Julia `memory_calibration` request rows from Rust-owned
/// calibration job inputs.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any host calibration job input
/// violates the staged `memory_calibration` request contract.
pub fn build_memory_calibration_request_rows_from_inputs(
    inputs: &[MemoryCalibrationInputs],
) -> Result<Vec<MemoryJuliaCalibrationRequestRow>, RepoIntelligenceError> {
    inputs.iter().map(build_request_row).collect()
}

/// Build one Julia `memory_calibration` request batch from Rust-owned
/// calibration job inputs.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the inputs are empty or any staged
/// row violates the Julia `memory_calibration` request contract.
pub fn build_memory_calibration_request_batch_from_inputs(
    inputs: &[MemoryCalibrationInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let rows = build_memory_calibration_request_rows_from_inputs(inputs)?;
    if rows.is_empty() {
        return Err(staging_error(
            "memory Julia memory_calibration host staging requires at least one input row",
        ));
    }
    build_memory_julia_calibration_request_batch(&rows)
}

fn build_request_row(
    input: &MemoryCalibrationInputs,
) -> Result<MemoryJuliaCalibrationRequestRow, RepoIntelligenceError> {
    Ok(MemoryJuliaCalibrationRequestRow {
        calibration_job_id: required_text(
            &input.calibration_job_id,
            "calibration_job_id",
            SURFACE,
        )?,
        scenario_pack: optional_text(input.scenario_pack.as_deref()),
        dataset_ref: required_text(&input.dataset_ref, "dataset_ref", SURFACE)?,
        objective: required_text(&input.objective, "objective", SURFACE)?,
        hyperparam_config: required_text(&input.hyperparam_config, "hyperparam_config", SURFACE)?,
    })
}

fn staging_error(message: impl Into<String>) -> RepoIntelligenceError {
    super::staging::staging_error(SURFACE, message)
}
