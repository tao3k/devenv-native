//! Batch processing helpers for Julia memory compute responses.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_runtime::{
    config::MemoryJuliaComputeRuntimeConfig,
    transport::{FLIGHT_SCHEMA_VERSION_METADATA_KEY, NegotiatedFlightTransportClient},
};

use crate::arrow_metadata::attach_record_batch_metadata;
use crate::memory::{
    MemoryJuliaComputeProfile, build_memory_julia_compute_flight_transport_client,
    validate_memory_julia_calibration_request_batches,
    validate_memory_julia_calibration_response_batches,
    validate_memory_julia_episodic_recall_request_batches,
    validate_memory_julia_episodic_recall_response_batches,
    validate_memory_julia_gate_score_request_batches,
    validate_memory_julia_gate_score_response_batches,
    validate_memory_julia_plan_tuning_request_batches,
    validate_memory_julia_plan_tuning_response_batches,
};

/// Validate staged memory-family request batches for one profile.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any batch violates the staged
/// request contract for the selected profile.
pub fn validate_memory_julia_compute_request_batches(
    profile: MemoryJuliaComputeProfile,
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    match profile {
        MemoryJuliaComputeProfile::EpisodicRecall => {
            validate_memory_julia_episodic_recall_request_batches(batches)
        }
        MemoryJuliaComputeProfile::MemoryGateScore => {
            validate_memory_julia_gate_score_request_batches(batches)
        }
        MemoryJuliaComputeProfile::MemoryPlanTuning => {
            validate_memory_julia_plan_tuning_request_batches(batches)
        }
        MemoryJuliaComputeProfile::MemoryCalibration => {
            validate_memory_julia_calibration_request_batches(batches)
        }
    }
}

/// Validate staged memory-family response batches for one profile.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any batch violates the staged
/// response contract for the selected profile.
pub fn validate_memory_julia_compute_response_batches(
    profile: MemoryJuliaComputeProfile,
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    match profile {
        MemoryJuliaComputeProfile::EpisodicRecall => {
            validate_memory_julia_episodic_recall_response_batches(batches)
        }
        MemoryJuliaComputeProfile::MemoryGateScore => {
            validate_memory_julia_gate_score_response_batches(batches)
        }
        MemoryJuliaComputeProfile::MemoryPlanTuning => {
            validate_memory_julia_plan_tuning_response_batches(batches)
        }
        MemoryJuliaComputeProfile::MemoryCalibration => {
            validate_memory_julia_calibration_response_batches(batches)
        }
    }
}

/// Send staged memory-family request batches through one negotiated Flight
/// client and validate the staged response contract.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request violates the staged
/// contract, request metadata cannot be attached, the Flight roundtrip fails,
/// or the response violates the staged contract.
pub async fn process_memory_julia_compute_flight_batches(
    client: &NegotiatedFlightTransportClient,
    profile: MemoryJuliaComputeProfile,
    schema_version: &str,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    validate_memory_julia_compute_request_batches(profile, batches)?;
    let request_batches = batches
        .iter()
        .map(|batch| attach_schema_version_metadata(batch, schema_version, profile))
        .collect::<Result<Vec<_>, _>>()?;
    let response_batches = client
        .process_batches(&request_batches)
        .await
        .map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "memory Julia compute Flight request for profile `{}` failed: {error}",
                profile.profile_id()
            ),
        })?;
    validate_memory_julia_compute_response_batches(profile, response_batches.as_slice())?;
    Ok(response_batches)
}

/// Build a runtime-config-driven Flight client for one memory-family profile,
/// execute the request batches, and validate the staged response contract.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the runtime is disabled, cannot be
/// negotiated into a client, the request violates the staged contract, or the
/// response fails validation.
pub async fn process_memory_julia_compute_flight_batches_for_runtime(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    let client =
        build_memory_julia_compute_flight_transport_client(runtime, profile)?.ok_or_else(|| {
            RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "memory Julia compute runtime is disabled or unavailable for profile `{}`",
                    profile.profile_id()
                ),
            }
        })?;
    process_memory_julia_compute_flight_batches(&client, profile, &runtime.schema_version, batches)
        .await
}

fn attach_schema_version_metadata(
    batch: &RecordBatch,
    schema_version: &str,
    profile: MemoryJuliaComputeProfile,
) -> Result<RecordBatch, RepoIntelligenceError> {
    attach_record_batch_metadata(
        batch,
        [(FLIGHT_SCHEMA_VERSION_METADATA_KEY, schema_version)],
    )
    .map_err(|error| RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "failed to attach memory Julia compute schema metadata for profile `{}`: {error}",
            profile.profile_id()
        ),
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/memory/transport/process.rs"]
mod tests;
