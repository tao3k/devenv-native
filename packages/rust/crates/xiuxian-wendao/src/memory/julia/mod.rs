//! Thin bridge from `xiuxian-wendao` into plugin-owned Julia memory compute.
//!
//! Ownership rule:
//! - `xiuxian-memory-engine` owns authoritative memory state and read models
//! - `xiuxian-wendao-runtime` owns runtime config and transport negotiation
//! - `xiuxian-julia-runtime` owns host staging, typed contracts, transport, and
//!   composed downcalls
//! - `xiuxian-wendao` exposes only this thin host-facing namespace

mod client;
mod runtime;

#[cfg(test)]
#[path = "../../../tests/unit/memory/julia/mod.rs"]
mod tests;

pub use client::ComputeClient;
pub use runtime::{resolve_memory_julia_compute_bindings, resolve_memory_julia_compute_runtime};
pub use xiuxian_julia_runtime::wendao::memory::host::{
    EpisodicRecallQueryInputs, MemoryCalibrationInputs, MemoryGateScoreEvidenceRow,
    MemoryLifecycleState, MemoryPlanTuningInputs, MemoryProjectionRow, MemoryUtilityLedger,
    RecallPlanTuning,
};
pub use xiuxian_julia_runtime::wendao::memory::{
    MemoryJuliaCalibrationArtifactRow, MemoryJuliaEpisodicRecallScoreRow,
    MemoryJuliaGateScoreRecommendationRow, MemoryJuliaPlanTuningAdviceRow,
};
pub use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;
