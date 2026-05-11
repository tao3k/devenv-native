//! Read-only projections from runtime-owned facts into polyglot contracts.

use xiuxian_polyglot_orchestrator::{
    AdmissionBudget, DoclingSchedulePlan, DoclingSchedulingInput, PolyglotControlSnapshot,
    PolyglotLane, PressureLevel, ReadinessState, RouteProfileRef, SnapshotInvariantError,
    WorkerPressureEvidence,
};

use crate::config::{MemoryJuliaComputeFallbackMode, MemoryJuliaComputeRuntimeConfig};
use crate::transport::ANALYSIS_DOCUMENT_EXTRACT_ROUTE;

/// Runtime facts used to project the Julia compute admission budget.
#[derive(Clone, Copy, Debug)]
pub struct MemoryJuliaComputeAdmissionBudgetInput<'a> {
    /// Runtime configuration resolved by the Wendao owner.
    pub config: &'a MemoryJuliaComputeRuntimeConfig,
    /// Currently active Julia compute requests.
    pub active_in_flight: u32,
    /// Queued Julia compute requests.
    pub queue_depth: u32,
    /// Current Julia readiness state.
    pub readiness: ReadinessState,
    /// Current Julia pressure level.
    pub pressure: PressureLevel,
}

/// Runtime facts used to project a neutral polyglot control snapshot.
#[derive(Clone, Copy, Debug)]
pub struct RuntimePolyglotSnapshotInput<'a> {
    /// Runtime configuration resolved by the Wendao owner.
    pub config: &'a MemoryJuliaComputeRuntimeConfig,
    /// Currently active Julia compute requests.
    pub active_in_flight: u32,
    /// Queued Julia compute requests.
    pub queue_depth: u32,
    /// Current Julia readiness state.
    pub readiness: ReadinessState,
    /// Current Julia pressure level.
    pub pressure: PressureLevel,
}

/// Owner-supplied pressure counters for document extraction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentExtractPressureEvidenceInput {
    /// Maximum in-flight worker count, when bounded.
    pub max_in_flight: Option<u32>,
    /// Currently active worker count.
    pub active_in_flight: u32,
    /// Queued shard count.
    pub queued_items: u32,
    /// Failed shard count.
    pub failed_items: u32,
    /// Retryable failure count.
    pub retryable_failures: u32,
    /// Whether the owner has a correctness-preserving fallback path.
    pub fallback_available: bool,
}

/// Returns a typed reference to the analyzer-owned document extraction route
/// known by runtime transport.
#[must_use]
pub fn document_extract_route_ref() -> RouteProfileRef {
    RouteProfileRef::document_extract(ANALYSIS_DOCUMENT_EXTRACT_ROUTE)
}

/// Returns the runtime-owned Julia compute admission budget.
#[must_use]
pub fn memory_julia_compute_admission_budget(
    input: MemoryJuliaComputeAdmissionBudgetInput<'_>,
) -> AdmissionBudget {
    AdmissionBudget {
        lane: PolyglotLane::JuliaCompute,
        max_in_flight: Some(max_in_flight_as_u32(input.config.max_in_flight_requests)),
        active_in_flight: input.active_in_flight,
        queue_depth: input.queue_depth,
        readiness: input.readiness,
        pressure: input.pressure,
        fallback_available: matches!(
            input.config.fallback_mode,
            MemoryJuliaComputeFallbackMode::Rust
        ),
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
    input: RuntimePolyglotSnapshotInput<'_>,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    PolyglotControlSnapshot::from_parts(
        vec![document_extract_route_ref()],
        vec![memory_julia_compute_admission_budget(
            MemoryJuliaComputeAdmissionBudgetInput {
                config: input.config,
                active_in_flight: input.active_in_flight,
                queue_depth: input.queue_depth,
                readiness: input.readiness,
                pressure: input.pressure,
            },
        )],
        Vec::new(),
    )
}

/// Returns document extraction pressure evidence from owner-supplied counters.
#[must_use]
pub fn document_extract_pressure_evidence(
    input: DocumentExtractPressureEvidenceInput,
) -> WorkerPressureEvidence {
    WorkerPressureEvidence::document_extraction()
        .with_worker_budget(input.max_in_flight, input.active_in_flight)
        .with_queue_depth(input.queued_items)
        .with_failures(input.failed_items, input.retryable_failures)
        .with_fallback_available(input.fallback_available)
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

fn max_in_flight_as_u32(max_in_flight_requests: u64) -> u32 {
    u32::try_from(max_in_flight_requests).unwrap_or(u32::MAX)
}

#[cfg(test)]
#[path = "../tests/unit/polyglot.rs"]
mod tests;
