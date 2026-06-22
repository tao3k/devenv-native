//! Host-level projections into polyglot scheduler contracts.
//!
//! `xiuxian-wendao` can see both the runtime-owned control-plane facts and
//! the orchestrator-owned graph/search profile facts. This module only aggregates
//! those read-only contracts; it does not dispatch work or change transport
//! schemas.

use xiuxian_polyglot_orchestrator::{
    MemoryJuliaComputeAdmissionBudgetInput, PolyglotControlSnapshot, PressureLevel, ReadinessState,
    SnapshotInvariantError, WendaoSearchLegacyRerankProfileRefInput, document_extract_route_ref,
    julia_graph_compute_profile_refs, memory_julia_compute_admission_budget,
    memory_julia_compute_profile_refs,
};
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

use crate::memory::julia::resolve_memory_julia_compute_runtime;
use crate::settings::{get_setting_string, merged_wendao_settings};

const LINK_GRAPH_LEGACY_RERANK_ROUTE_KEY: &str = "link_graph.retrieval.julia_rerank.route";
const LINK_GRAPH_LEGACY_RERANK_SCHEMA_VERSION_KEY: &str =
    "link_graph.retrieval.julia_rerank.schema_version";
const LINK_GRAPH_LEGACY_RERANK_ROUTE_ENV: &str = "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_ROUTE";
const LINK_GRAPH_LEGACY_RERANK_SCHEMA_VERSION_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION";

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
    let legacy_rerank_route = resolve_optional_setting_or_env(
        &settings,
        LINK_GRAPH_LEGACY_RERANK_ROUTE_KEY,
        LINK_GRAPH_LEGACY_RERANK_ROUTE_ENV,
    );
    let legacy_rerank_schema_version = resolve_optional_setting_or_env(
        &settings,
        LINK_GRAPH_LEGACY_RERANK_SCHEMA_VERSION_KEY,
        LINK_GRAPH_LEGACY_RERANK_SCHEMA_VERSION_ENV,
    );

    wendao_polyglot_control_snapshot_from_parts(
        &memory_runtime,
        WendaoSearchLegacyRerankProfileRefInput {
            route: legacy_rerank_route.as_deref(),
            schema_version: legacy_rerank_schema_version.as_deref(),
        },
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
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn wendao_polyglot_control_snapshot_from_parts(
    memory_runtime: &MemoryJuliaComputeRuntimeConfig,
    legacy_rerank: WendaoSearchLegacyRerankProfileRefInput<'_>,
    active_in_flight: u32,
    queue_depth: u32,
    readiness: ReadinessState,
    pressure: PressureLevel,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    let mut route_refs = vec![document_extract_route_ref()];
    route_refs.extend(memory_julia_compute_profile_refs(memory_runtime));
    route_refs.extend(julia_graph_compute_profile_refs(legacy_rerank));

    let admission_budget =
        memory_julia_compute_admission_budget(MemoryJuliaComputeAdmissionBudgetInput {
            config: memory_runtime,
            active_in_flight,
            queue_depth,
            readiness,
            pressure,
        });

    PolyglotControlSnapshot::from_parts(route_refs, vec![admission_budget], Vec::new())
}

fn resolve_optional_setting_or_env(
    settings: &serde_yaml::Value,
    dotted_key: &str,
    env_name: &str,
) -> Option<String> {
    resolve_optional_setting_or_env_with_lookup(settings, dotted_key, env_name, |name| {
        std::env::var(name).ok()
    })
}

fn resolve_optional_setting_or_env_with_lookup<F>(
    settings: &serde_yaml::Value,
    dotted_key: &str,
    env_name: &str,
    env_lookup: F,
) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    first_non_empty([
        get_setting_string(settings, dotted_key),
        env_lookup(env_name),
    ])
}

fn first_non_empty(values: [Option<String>; 2]) -> Option<String> {
    values.into_iter().find_map(|value| {
        let trimmed = value?.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[cfg(test)]
#[path = "../tests/unit/polyglot.rs"]
mod tests;
