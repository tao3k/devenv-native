//! Runtime profile helpers for Julia memory compute backends.

pub use crate::wendao::{
    MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_CALIBRATION_REQUEST_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_CALIBRATION_RESPONSE_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_REQUEST_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_RESPONSE_SCHEMA_ID, MEMORY_JULIA_COMPUTE_FAMILY_ID,
    MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID, MEMORY_JULIA_COMPUTE_GATE_SCORE_REQUEST_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_GATE_SCORE_RESPONSE_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_PLAN_TUNING_REQUEST_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_PLAN_TUNING_RESPONSE_SCHEMA_ID, MemoryJuliaComputeProfile,
};

/// Returns the stable request schema id for one memory-family profile.
pub(crate) const fn request_schema_id(profile: MemoryJuliaComputeProfile) -> &'static str {
    match profile {
        MemoryJuliaComputeProfile::EpisodicRecall => {
            MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_REQUEST_SCHEMA_ID
        }
        MemoryJuliaComputeProfile::MemoryGateScore => {
            MEMORY_JULIA_COMPUTE_GATE_SCORE_REQUEST_SCHEMA_ID
        }
        MemoryJuliaComputeProfile::MemoryPlanTuning => {
            MEMORY_JULIA_COMPUTE_PLAN_TUNING_REQUEST_SCHEMA_ID
        }
        MemoryJuliaComputeProfile::MemoryCalibration => {
            MEMORY_JULIA_COMPUTE_CALIBRATION_REQUEST_SCHEMA_ID
        }
    }
}

/// Returns the stable response schema id for one memory-family profile.
pub(crate) const fn response_schema_id(profile: MemoryJuliaComputeProfile) -> &'static str {
    match profile {
        MemoryJuliaComputeProfile::EpisodicRecall => {
            MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_RESPONSE_SCHEMA_ID
        }
        MemoryJuliaComputeProfile::MemoryGateScore => {
            MEMORY_JULIA_COMPUTE_GATE_SCORE_RESPONSE_SCHEMA_ID
        }
        MemoryJuliaComputeProfile::MemoryPlanTuning => {
            MEMORY_JULIA_COMPUTE_PLAN_TUNING_RESPONSE_SCHEMA_ID
        }
        MemoryJuliaComputeProfile::MemoryCalibration => {
            MEMORY_JULIA_COMPUTE_CALIBRATION_RESPONSE_SCHEMA_ID
        }
    }
}
