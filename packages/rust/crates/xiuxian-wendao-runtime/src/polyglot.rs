//! Read-only projections from runtime-owned facts into polyglot contracts.

use xiuxian_polyglot_orchestrator::{
    AdmissionBudget, DoclingSchedulePlan, DoclingSchedulingInput, PolyglotControlSnapshot,
    PolyglotLane, PressureLevel, ReadinessState, RouteProfileRef, SnapshotInvariantError,
    WorkerPressureEvidence,
};

use crate::config::{MemoryJuliaComputeFallbackMode, MemoryJuliaComputeRuntimeConfig};
use crate::transport::ANALYSIS_DOCUMENT_EXTRACT_ROUTE;

/// Returns a typed reference to the analyzer-owned document extraction route
/// known by runtime transport.
#[must_use]
pub fn document_extract_route_ref() -> RouteProfileRef {
    RouteProfileRef::document_extract(ANALYSIS_DOCUMENT_EXTRACT_ROUTE)
}

/// Returns the runtime-owned Julia compute admission budget.
#[must_use]
pub fn memory_julia_compute_admission_budget(
    config: &MemoryJuliaComputeRuntimeConfig,
    active_in_flight: u32,
    queue_depth: u32,
    readiness: ReadinessState,
    pressure: PressureLevel,
) -> AdmissionBudget {
    AdmissionBudget {
        lane: PolyglotLane::JuliaCompute,
        max_in_flight: Some(max_in_flight_as_u32(config.max_in_flight_requests)),
        active_in_flight,
        queue_depth,
        readiness,
        pressure,
        fallback_available: matches!(config.fallback_mode, MemoryJuliaComputeFallbackMode::Rust),
    }
}

/// Builds a read-only runtime control-plane snapshot from already supplied
/// runtime facts.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
pub fn runtime_polyglot_snapshot(
    config: &MemoryJuliaComputeRuntimeConfig,
    active_in_flight: u32,
    queue_depth: u32,
    readiness: ReadinessState,
    pressure: PressureLevel,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    PolyglotControlSnapshot::from_parts(
        vec![document_extract_route_ref()],
        vec![memory_julia_compute_admission_budget(
            config,
            active_in_flight,
            queue_depth,
            readiness,
            pressure,
        )],
        Vec::new(),
    )
}

/// Returns document extraction pressure evidence from owner-supplied counters.
#[must_use]
pub fn document_extract_pressure_evidence(
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queued_items: u32,
    failed_items: u32,
    retryable_failures: u32,
    fallback_available: bool,
) -> WorkerPressureEvidence {
    WorkerPressureEvidence::document_extraction()
        .with_worker_budget(max_in_flight, active_in_flight)
        .with_queue_depth(queued_items)
        .with_failures(failed_items, retryable_failures)
        .with_fallback_available(fallback_available)
}

/// Builds a read-only document extraction pressure snapshot from supplied
/// counters.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
pub fn document_extract_pressure_snapshot(
    pressure: WorkerPressureEvidence,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    PolyglotControlSnapshot::from_parts(
        vec![document_extract_route_ref()],
        vec![pressure.to_admission_budget()],
        vec![pressure.to_lane_evidence()],
    )
}

/// Returns an inert document extraction scheduling plan from runtime-owned
/// pressure facts and caller-local worker bounds.
#[must_use]
pub fn document_extract_schedule_plan(
    pressure: WorkerPressureEvidence,
    requested_workers: Option<u32>,
    max_worker_cap: Option<u32>,
    shard_count: u32,
) -> DoclingSchedulePlan {
    DoclingSchedulingInput::document_extraction(pressure)
        .with_worker_request(requested_workers, max_worker_cap)
        .with_shard_count(shard_count)
        .plan()
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
