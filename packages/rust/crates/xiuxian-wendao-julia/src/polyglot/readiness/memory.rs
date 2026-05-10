//! Memory-family Julia readiness and scheduling projections.

use xiuxian_polyglot_orchestrator::{
    BenchmarkState, ContractValidationState, FallbackEvidence, HealthState, JuliaComputeTaskShape,
    JuliaReadinessEvidence, JuliaSchedulePlan, LaneEvidence, ManifestReadinessState,
    PolyglotControlSnapshot, PolyglotLane, PressureLevel, ReadinessState, SnapshotInvariantError,
    WarmupState,
};
use xiuxian_wendao_runtime::config::{
    MemoryJuliaComputeFallbackMode, MemoryJuliaComputeRuntimeConfig,
};

use crate::memory::MemoryJuliaComputeProfile;
use crate::polyglot::state::{
    JuliaProfileSchedulingFacts, memory_julia_compute_profile_ref,
    memory_julia_compute_profile_refs,
};

use super::evidence_support::{julia_schedule_plan_from_readiness, max_in_flight_as_u32};

/// Named readiness input for one memory-family Julia compute profile.
#[derive(Clone, Copy, Debug)]
pub struct MemoryJuliaComputeReadinessInput<'a> {
    /// Runtime config that owns route/schema/fallback facts.
    pub runtime: &'a MemoryJuliaComputeRuntimeConfig,
    /// Memory compute profile being described.
    pub profile: MemoryJuliaComputeProfile,
    /// Julia warmup state observed by the owner.
    pub warmup: WarmupState,
    /// Benchmark state observed by the owner.
    pub benchmark: BenchmarkState,
    /// Active in-flight request count.
    pub active_in_flight: u32,
    /// Queued request count.
    pub queue_depth: u32,
}

/// Builds a read-only snapshot for memory-family Julia compute contracts.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
pub fn memory_julia_compute_snapshot(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    PolyglotControlSnapshot::from_parts(
        memory_julia_compute_profile_refs(runtime),
        Vec::new(),
        vec![LaneEvidence::new(
            PolyglotLane::JuliaCompute,
            HealthState::Unknown,
            memory_julia_compute_config_readiness(runtime),
            PressureLevel::Unknown,
            FallbackEvidence::new(false),
        )],
    )
}

/// Returns Julia readiness evidence for one memory-family profile.
///
/// This derives route/profile/schema facts from the runtime config and accepts
/// warmup, benchmark, and admission-window facts supplied by the owner. It does
/// not probe, warm up, or schedule a Julia worker.
#[must_use]
pub fn memory_julia_compute_readiness_evidence(
    input: MemoryJuliaComputeReadinessInput<'_>,
) -> JuliaReadinessEvidence {
    let MemoryJuliaComputeReadinessInput {
        runtime,
        profile,
        warmup,
        benchmark,
        active_in_flight,
        queue_depth,
    } = input;
    let contract = profile.contract();
    let schema_validation = if runtime.schema_version.is_empty() {
        ContractValidationState::Invalid
    } else {
        ContractValidationState::Valid
    };

    JuliaReadinessEvidence::memory_profile(contract.profile_id)
        .with_schema_version(runtime.schema_version.as_str())
        .with_route_validation(config_route_validation(runtime, profile))
        .with_schema_validation(schema_validation)
        .with_manifest_readiness(config_manifest_readiness(runtime))
        .with_warmup(warmup)
        .with_benchmark(benchmark)
        .with_admission_window(
            Some(max_in_flight_as_u32(runtime.max_in_flight_requests)),
            active_in_flight,
            queue_depth,
        )
        .with_fallback_available(matches!(
            runtime.fallback_mode,
            MemoryJuliaComputeFallbackMode::Rust
        ))
}

/// Returns a schedule plan for one memory-family Julia compute profile.
#[must_use]
pub fn memory_julia_compute_schedule_plan(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let fallback_available = facts.fallback_available
        || matches!(runtime.fallback_mode, MemoryJuliaComputeFallbackMode::Rust);
    let facts = facts.with_fallback_available(fallback_available);
    let readiness = memory_julia_compute_readiness_evidence(MemoryJuliaComputeReadinessInput {
        runtime,
        profile,
        warmup: facts.runtime_stats.warmup,
        benchmark: facts.runtime_stats.benchmark,
        active_in_flight: facts.runtime_stats.active_in_flight,
        queue_depth: facts.runtime_stats.queue_depth,
    })
    .with_fallback_available(fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
}

/// Builds a read-only snapshot for one memory-family Julia readiness profile.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
pub fn memory_julia_compute_readiness_snapshot(
    input: MemoryJuliaComputeReadinessInput<'_>,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    let runtime = input.runtime;
    let profile = input.profile;
    let evidence = memory_julia_compute_readiness_evidence(input);
    PolyglotControlSnapshot::from_parts(
        vec![memory_julia_compute_profile_ref(runtime, profile)],
        vec![evidence.to_admission_budget()],
        vec![evidence.to_lane_evidence()],
    )
}

/// Returns config-level readiness for the memory-family Julia compute lane.
///
/// This does not perform a live health probe. Runtime process readiness belongs
/// to later slices and should feed this contract with separate evidence.
#[must_use]
pub const fn memory_julia_compute_config_readiness(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> ReadinessState {
    if runtime.enabled {
        ReadinessState::Ready
    } else {
        ReadinessState::Disabled
    }
}

const fn config_manifest_readiness(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> ManifestReadinessState {
    if runtime.enabled {
        ManifestReadinessState::Ready
    } else {
        ManifestReadinessState::Missing
    }
}

fn config_route_validation(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> ContractValidationState {
    if route_for_profile(runtime, profile).is_empty() {
        ContractValidationState::Invalid
    } else {
        ContractValidationState::Valid
    }
}

fn route_for_profile(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> &str {
    match profile {
        MemoryJuliaComputeProfile::EpisodicRecall => runtime.routes.episodic_recall.as_str(),
        MemoryJuliaComputeProfile::MemoryGateScore => runtime.routes.memory_gate_score.as_str(),
        MemoryJuliaComputeProfile::MemoryPlanTuning => runtime.routes.memory_plan_tuning.as_str(),
        MemoryJuliaComputeProfile::MemoryCalibration => runtime.routes.memory_calibration.as_str(),
    }
}
