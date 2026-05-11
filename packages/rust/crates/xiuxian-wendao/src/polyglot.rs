//! Host-level projections into polyglot scheduler contracts.
//!
//! `xiuxian-wendao` can see both the runtime-owned control-plane facts and
//! the Julia-owned graph/search profile facts. This module only aggregates
//! those read-only contracts; it does not dispatch work or change transport
//! schemas.

use xiuxian_polyglot_orchestrator::{
    PolyglotControlSnapshot, PressureLevel, ReadinessState, SnapshotInvariantError,
};
use xiuxian_wendao_julia::compatibility::link_graph::LinkGraphJuliaRerankRuntimeConfig;
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

use crate::memory::julia::resolve_memory_julia_compute_runtime;
use crate::settings::merged_wendao_settings;

/// Resolve the host-level polyglot control snapshot from current Wendao
/// settings and caller-supplied Julia lane counters.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] when the assembled route/profile refs or
/// admission facts violate neutral orchestrator invariants.
pub fn resolve_wendao_polyglot_control_snapshot(
    active_in_flight: u32,
    queue_depth: u32,
    readiness: ReadinessState,
    pressure: PressureLevel,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    let memory_runtime = resolve_memory_julia_compute_runtime();
    let settings = merged_wendao_settings();
    let link_graph_julia_runtime =
        LinkGraphJuliaRerankRuntimeConfig::resolve_with_settings(&settings);

    wendao_polyglot_control_snapshot_from_parts(
        &memory_runtime,
        &link_graph_julia_runtime,
        active_in_flight,
        queue_depth,
        readiness,
        pressure,
    )
}

/// Build a host-level polyglot control snapshot from already resolved owner
/// facts.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] when the assembled route/profile refs or
/// admission facts violate neutral orchestrator invariants.
pub fn wendao_polyglot_control_snapshot_from_parts(
    memory_runtime: &MemoryJuliaComputeRuntimeConfig,
    link_graph_julia_runtime: &LinkGraphJuliaRerankRuntimeConfig,
    active_in_flight: u32,
    queue_depth: u32,
    readiness: ReadinessState,
    pressure: PressureLevel,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    let mut route_refs = vec![xiuxian_wendao_runtime::polyglot::document_extract_route_ref()];
    route_refs
        .extend(xiuxian_wendao_julia::polyglot::memory_julia_compute_profile_refs(memory_runtime));
    route_refs.extend(
        xiuxian_wendao_julia::polyglot::julia_graph_compute_profile_refs(link_graph_julia_runtime),
    );

    let admission_budget = xiuxian_wendao_runtime::polyglot::memory_julia_compute_admission_budget(
        xiuxian_wendao_runtime::polyglot::MemoryJuliaComputeAdmissionBudgetInput {
            config: memory_runtime,
            active_in_flight,
            queue_depth,
            readiness,
            pressure,
        },
    );

    PolyglotControlSnapshot::from_parts(route_refs, vec![admission_budget], Vec::new())
}

#[cfg(test)]
#[path = "../tests/unit/polyglot.rs"]
mod tests;
