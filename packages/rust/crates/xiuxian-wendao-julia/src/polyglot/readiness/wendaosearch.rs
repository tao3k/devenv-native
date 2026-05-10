//! `WendaoSearch` readiness and scheduling projections.

use xiuxian_polyglot_orchestrator::{
    BenchmarkState, ContractValidationState, JuliaComputeTaskShape, JuliaReadinessEvidence,
    JuliaSchedulePlan, JuliaThreadPinningDiagnostics, LaneCapability, ManifestReadinessState,
    WarmupState,
};

use crate::compatibility::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, LinkGraphJuliaRerankRuntimeConfig,
};
use crate::polyglot::state::{
    JuliaProfileSchedulingFacts, WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
    wendaosearch_graph_structural_profile_id,
};
use crate::{GraphStructuralRouteKind, JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION};

use super::evidence_support::{
    JuliaReadinessWindow, JuliaStaticContractReadinessProfile, julia_schedule_plan_from_readiness,
    julia_static_contract_readiness_evidence,
};

/// Readiness facts for one `WendaoSearch.jl` graph-structural route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WendaoSearchGraphStructuralReadinessInput {
    /// Graph-structural route kind being described.
    pub route_kind: GraphStructuralRouteKind,
    /// Julia warmup state observed by the owner.
    pub warmup: WarmupState,
    /// Benchmark state observed by the owner.
    pub benchmark: BenchmarkState,
    /// Optional maximum concurrent request budget.
    pub max_in_flight: Option<u32>,
    /// Active in-flight request count.
    pub active_in_flight: u32,
    /// Queued request count.
    pub queue_depth: u32,
}

/// Readiness facts for the legacy `WendaoSearch.jl` rerank route.
#[derive(Clone, Copy, Debug)]
pub struct WendaoSearchLegacyRerankReadinessInput<'a> {
    /// Runtime config that owns route/schema facts.
    pub runtime: &'a LinkGraphJuliaRerankRuntimeConfig,
    /// Julia warmup state observed by the owner.
    pub warmup: WarmupState,
    /// Benchmark state observed by the owner.
    pub benchmark: BenchmarkState,
    /// Active in-flight request count.
    pub active_in_flight: u32,
    /// Queued request count.
    pub queue_depth: u32,
}

/// Returns readiness evidence for one `WendaoSearch.jl` graph-structural
/// profile.
#[must_use]
pub fn wendaosearch_graph_structural_readiness_evidence(
    input: WendaoSearchGraphStructuralReadinessInput,
) -> JuliaReadinessEvidence {
    julia_static_contract_readiness_evidence(
        JuliaStaticContractReadinessProfile {
            capability: LaneCapability::GraphSearchCompute,
            profile_id: wendaosearch_graph_structural_profile_id(input.route_kind),
            schema_version: JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION,
        },
        input.warmup,
        input.benchmark,
        JuliaReadinessWindow {
            max_in_flight: input.max_in_flight,
            active_in_flight: input.active_in_flight,
            queue_depth: input.queue_depth,
        },
    )
}

/// Returns a schedule plan for one `WendaoSearch.jl` graph-structural profile.
#[must_use]
pub fn wendaosearch_graph_structural_schedule_plan(
    route_kind: GraphStructuralRouteKind,
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let readiness = wendaosearch_graph_structural_readiness_evidence(
        WendaoSearchGraphStructuralReadinessInput {
            route_kind,
            warmup: facts.runtime_stats.warmup,
            benchmark: facts.runtime_stats.benchmark,
            max_in_flight: facts.max_in_flight,
            active_in_flight: facts.runtime_stats.active_in_flight,
            queue_depth: facts.runtime_stats.queue_depth,
        },
    )
    .with_fallback_available(facts.fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
}

/// Attaches Julia-owned thread-pinning diagnostics to readiness evidence.
///
/// The diagnostics are observability facts only. They do not change route,
/// schema, warmup, benchmark, admission, or fallback state.
#[must_use]
pub fn with_julia_thread_pinning_diagnostics(
    evidence: JuliaReadinessEvidence,
    diagnostics: JuliaThreadPinningDiagnostics,
) -> JuliaReadinessEvidence {
    evidence.with_thread_pinning_diagnostics(diagnostics)
}

/// Returns readiness evidence for the legacy `WendaoSearch.jl` rerank profile.
#[must_use]
pub fn wendaosearch_legacy_rerank_readiness_evidence(
    input: WendaoSearchLegacyRerankReadinessInput<'_>,
) -> JuliaReadinessEvidence {
    let WendaoSearchLegacyRerankReadinessInput {
        runtime,
        warmup,
        benchmark,
        active_in_flight,
        queue_depth,
    } = input;
    let route_validation = if runtime
        .route
        .as_deref()
        .unwrap_or(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE)
        .is_empty()
    {
        ContractValidationState::Invalid
    } else {
        ContractValidationState::Valid
    };
    let schema_validation = match runtime.schema_version.as_deref() {
        Some("") => ContractValidationState::Invalid,
        _ => ContractValidationState::Valid,
    };

    JuliaReadinessEvidence::graph_search_profile(WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID)
        .with_schema_version(runtime.schema_version.as_deref().unwrap_or("v1"))
        .with_route_validation(route_validation)
        .with_schema_validation(schema_validation)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(warmup)
        .with_benchmark(benchmark)
        .with_admission_window(None, active_in_flight, queue_depth)
        .with_fallback_available(false)
}

/// Returns a schedule plan for the legacy `WendaoSearch.jl` rerank profile.
#[must_use]
pub fn wendaosearch_legacy_rerank_schedule_plan(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let readiness =
        wendaosearch_legacy_rerank_readiness_evidence(WendaoSearchLegacyRerankReadinessInput {
            runtime,
            warmup: facts.runtime_stats.warmup,
            benchmark: facts.runtime_stats.benchmark,
            active_in_flight: facts.runtime_stats.active_in_flight,
            queue_depth: facts.runtime_stats.queue_depth,
        })
        .with_admission_window(
            facts.max_in_flight,
            facts.runtime_stats.active_in_flight,
            facts.runtime_stats.queue_depth,
        )
        .with_fallback_available(facts.fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
}
