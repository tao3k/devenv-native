//! Read-only projections from Julia-owned facts into polyglot contracts.

use xiuxian_polyglot_orchestrator::{
    BenchmarkState, ContractValidationState, FallbackEvidence, HealthState, JuliaReadinessEvidence,
    LaneEvidence, ManifestReadinessState, PolyglotControlSnapshot, PolyglotLane, PressureLevel,
    ReadinessState, RouteProfileRef, SnapshotInvariantError, WarmupState,
};
use xiuxian_wendao_runtime::config::{
    MemoryJuliaComputeFallbackMode, MemoryJuliaComputeRuntimeConfig,
};

use crate::memory::{
    MemoryJuliaComputeManifestRow, MemoryJuliaComputeProfile,
    build_memory_julia_compute_manifest_rows,
};

/// Returns typed refs for every staged memory-family Julia compute profile.
#[must_use]
pub fn memory_julia_compute_profile_refs(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> Vec<RouteProfileRef> {
    build_memory_julia_compute_manifest_rows(runtime)
        .iter()
        .map(memory_julia_compute_manifest_row_ref)
        .collect()
}

/// Returns a typed ref for one staged memory-family Julia compute profile.
#[must_use]
pub fn memory_julia_compute_profile_ref(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> RouteProfileRef {
    let contract = profile.contract();
    RouteProfileRef::julia_profile(
        route_for_profile(runtime, profile),
        contract.profile_id,
        runtime.schema_version.as_str(),
    )
}

/// Returns a typed ref from an already materialized Julia memory manifest row.
#[must_use]
pub fn memory_julia_compute_manifest_row_ref(
    row: &MemoryJuliaComputeManifestRow,
) -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        row.route.as_str(),
        row.profile_id.as_str(),
        row.schema_version.as_str(),
    )
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
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
    warmup: WarmupState,
    benchmark: BenchmarkState,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
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

/// Builds a read-only snapshot for one memory-family Julia readiness profile.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
pub fn memory_julia_compute_readiness_snapshot(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
    warmup: WarmupState,
    benchmark: BenchmarkState,
    active_in_flight: u32,
    queue_depth: u32,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    let evidence = memory_julia_compute_readiness_evidence(
        runtime,
        profile,
        warmup,
        benchmark,
        active_in_flight,
        queue_depth,
    );
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

const fn max_in_flight_as_u32(max_in_flight_requests: u64) -> u32 {
    if max_in_flight_requests > u32::MAX as u64 {
        u32::MAX
    } else {
        max_in_flight_requests as u32
    }
}

#[cfg(test)]
#[path = "../tests/unit/polyglot.rs"]
mod tests;
