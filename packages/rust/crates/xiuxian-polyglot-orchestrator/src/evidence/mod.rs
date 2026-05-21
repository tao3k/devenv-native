//! Health, readiness, pressure, and fallback evidence contracts.

mod model;

pub use model::{
    FallbackEvidence, HealthState, LaneEvidence, LaneEvidenceInput, PressureLevel, ReadinessState,
};
