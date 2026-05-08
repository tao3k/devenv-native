//! Read-only projections from Julia-owned facts into polyglot contracts.

use xiuxian_polyglot_orchestrator::{
    BenchmarkState, ContractValidationState, FallbackEvidence, HealthState,
    JuliaAcceleratorDiagnostics, JuliaComputeTaskShape, JuliaReadinessEvidence, JuliaRuntimeStats,
    JuliaSchedulePlan, JuliaSchedulingInput, JuliaThreadPinningDiagnostics, LaneCapability,
    LaneEvidence, ManifestReadinessState, PolyglotControlSnapshot, PolyglotLane, PressureLevel,
    ReadinessState, RouteProfileRef, SnapshotInvariantError, WarmupState,
};
use xiuxian_wendao_runtime::config::{
    MemoryJuliaComputeFallbackMode, MemoryJuliaComputeRuntimeConfig,
};

use crate::compatibility::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, LinkGraphJuliaRerankRuntimeConfig,
};
use crate::integration_support::{
    WendaoGraphGnnHostProbeReport, WendaoGraphLinkGraphFullStructuralHostProbeReport,
    WendaoGraphLinkGraphHostProbeReport, WendaoGraphPageIndexHostProbeReport,
    WendaoGraphPageIndexPlannerActionHostProbeReport,
};
use crate::memory::{
    MemoryJuliaComputeManifestRow, MemoryJuliaComputeProfile,
    build_memory_julia_compute_manifest_rows,
};
use crate::{
    GraphStructuralRouteKind, JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION,
    WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
};

/// Stable profile id for the `WendaoGraph.jl` link-evidence contract.
pub const WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID: &str = "wendao_graph_link_evidence";
/// Stable profile id for the `WendaoGraph.jl` `PageIndex` reasoning contract.
pub const WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID: &str = "wendao_graph_page_index_reasoning";
/// Host-entrypoint identifier for local `WendaoGraph.jl` `PageIndex` reasoning.
pub const WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT: &str =
    "WendaoGraph.page_index_reasoning_from_request";
/// Stable profile id for the `WendaoGraph.jl` GNN reasoning contract.
pub const WENDAO_GRAPH_GNN_REASONING_PROFILE_ID: &str = "wendao_graph_gnn_reasoning";
/// Host-entrypoint identifier for local `WendaoGraph.jl` GNN reasoning.
pub const WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT: &str = "WendaoGraph.gnn_node_scores";
/// Contract version for the `WendaoGraph.jl` GNN host-probe evidence surface.
pub const WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION: &str = "wendaograph-gnn-host-probe-v1";
/// Stable profile id for the legacy `WendaoSearch.jl` rerank route.
pub const WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID: &str = "wendaosearch_legacy_rerank";
/// Stable profile id for the `WendaoSearch.jl` structural-rerank route.
pub const WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID: &str = "wendaosearch_structural_rerank";
/// Stable profile id for the `WendaoSearch.jl` constraint-filter route.
pub const WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID: &str = "wendaosearch_constraint_filter";

mod wendaograph_algorithms;

pub use wendaograph_algorithms::{
    WendaoGraphAlgorithmRef, WendaoGraphAlgorithmWorkload, wendaograph_algorithm_ref,
    wendaograph_algorithm_refs, wendaograph_algorithm_refs_for_profile,
    wendaograph_algorithm_task_shape, wendaograph_frontier_algorithm_ref,
    wendaograph_frontier_task_shape, wendaograph_gnn_algorithm_refs,
    wendaograph_link_graph_algorithm_refs, wendaograph_page_index_algorithm_refs,
    wendaograph_relationship_search_algorithm_refs,
    wendaograph_search_strategy_flow_algorithm_refs,
};

/// Owner-supplied scheduling facts for one Julia profile planning attempt.
///
/// These facts are inert. They do not start Julia, probe a worker, mutate a
/// queue, or execute Rust fallback code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JuliaProfileSchedulingFacts {
    /// Optional maximum number of in-flight Julia requests for this profile.
    pub max_in_flight: Option<u32>,
    /// Runtime stats supplied by the owner package or host.
    pub runtime_stats: JuliaRuntimeStats,
    /// Whether an owner-defined Rust fallback is safe for this task.
    pub fallback_available: bool,
    /// Optional hard deadline in milliseconds.
    pub deadline_ms: Option<u32>,
    /// Optional target latency in milliseconds.
    pub target_latency_ms: Option<u32>,
}

/// Relationship-search scheduling evidence projected from a `WendaoGraph.jl`
/// host probe.
///
/// This is descriptive owner evidence. It does not call Julia, add a route, or
/// turn row counts into hard admission gates.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphRelationshipSearchEvidence {
    /// Catalog entry covered by this evidence row.
    pub algorithm: WendaoGraphAlgorithmRef,
    /// Host-probe table that backs this relationship-search algorithm row.
    pub probe_table: Option<&'static str>,
    /// Row count observed in the host-probe backing table.
    pub probe_rows: Option<u32>,
    /// Runtime stats projected from the host-probe timing report.
    pub runtime_stats: JuliaRuntimeStats,
    /// Schedule plan produced by the existing algorithm schedule helper.
    pub schedule_plan: JuliaSchedulePlan,
}

impl JuliaProfileSchedulingFacts {
    /// Creates scheduling facts from observed or inferred runtime stats.
    #[must_use]
    pub const fn new(runtime_stats: JuliaRuntimeStats) -> Self {
        Self {
            max_in_flight: None,
            runtime_stats,
            fallback_available: false,
            deadline_ms: None,
            target_latency_ms: None,
        }
    }

    /// Returns these facts with an admission capacity override.
    #[must_use]
    pub const fn with_max_in_flight(mut self, max_in_flight: Option<u32>) -> Self {
        self.max_in_flight = max_in_flight;
        self
    }

    /// Returns these facts with fallback availability.
    #[must_use]
    pub const fn with_fallback_available(mut self, fallback_available: bool) -> Self {
        self.fallback_available = fallback_available;
        self
    }

    /// Returns these facts with a hard deadline in milliseconds.
    #[must_use]
    pub const fn with_deadline_ms(mut self, deadline_ms: Option<u32>) -> Self {
        self.deadline_ms = deadline_ms;
        self
    }

    /// Returns these facts with a target latency in milliseconds.
    #[must_use]
    pub const fn with_target_latency_ms(mut self, target_latency_ms: Option<u32>) -> Self {
        self.target_latency_ms = target_latency_ms;
        self
    }
}

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

/// Returns the typed ref for the `WendaoGraph.jl` link-evidence contract.
#[must_use]
pub fn wendao_graph_link_evidence_profile_ref() -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
        WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    )
}

/// Returns the typed ref for the `WendaoGraph.jl` `PageIndex` reasoning contract.
#[must_use]
pub fn wendao_graph_page_index_reasoning_profile_ref() -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
        WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    )
}

/// Returns the typed ref for the `WendaoGraph.jl` GNN reasoning contract.
#[must_use]
pub fn wendao_graph_gnn_reasoning_profile_ref() -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT,
        WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
        WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
    )
}

/// Returns a typed ref for one `WendaoSearch.jl` graph-structural route.
#[must_use]
pub fn wendaosearch_graph_structural_profile_ref(
    route_kind: GraphStructuralRouteKind,
) -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        route_kind.route(),
        wendaosearch_graph_structural_profile_id(route_kind),
        route_kind.schema_version(),
    )
}

/// Returns typed refs for the staged `WendaoSearch.jl` graph-structural routes.
#[must_use]
pub fn wendaosearch_graph_structural_profile_refs() -> Vec<RouteProfileRef> {
    [
        GraphStructuralRouteKind::StructuralRerank,
        GraphStructuralRouteKind::ConstraintFilter,
    ]
    .into_iter()
    .map(wendaosearch_graph_structural_profile_ref)
    .collect()
}

/// Returns the typed ref for the legacy `WendaoSearch.jl` rerank route.
#[must_use]
pub fn wendaosearch_legacy_rerank_profile_ref(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        runtime
            .route
            .as_deref()
            .unwrap_or(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE),
        WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
        runtime.schema_version.as_deref().unwrap_or("v1"),
    )
}

/// Returns graph-family Julia route refs currently known to the Rust scheduler.
#[must_use]
pub fn julia_graph_compute_profile_refs(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> Vec<RouteProfileRef> {
    let mut refs = Vec::with_capacity(6);
    refs.push(wendao_graph_link_evidence_profile_ref());
    refs.push(wendao_graph_page_index_reasoning_profile_ref());
    refs.push(wendao_graph_gnn_reasoning_profile_ref());
    refs.push(wendaosearch_legacy_rerank_profile_ref(runtime));
    refs.extend(wendaosearch_graph_structural_profile_refs());
    refs
}

/// Builds a read-only graph-family Julia contract snapshot.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
pub fn julia_graph_compute_snapshot(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    PolyglotControlSnapshot::from_parts(
        julia_graph_compute_profile_refs(runtime),
        Vec::new(),
        Vec::new(),
    )
}

/// Returns readiness evidence for the `WendaoGraph.jl` link-evidence profile.
#[must_use]
pub fn wendao_graph_link_evidence_readiness_evidence(
    warmup: WarmupState,
    benchmark: BenchmarkState,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    julia_static_contract_readiness_evidence(
        JuliaStaticContractReadinessProfile {
            capability: LaneCapability::GraphEvidenceCompute,
            profile_id: WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
            schema_version: WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
        },
        warmup,
        benchmark,
        JuliaReadinessWindow {
            max_in_flight,
            active_in_flight,
            queue_depth,
        },
    )
}

/// Returns a schedule plan for the `WendaoGraph.jl` link-evidence profile.
#[must_use]
pub fn wendao_graph_link_evidence_schedule_plan(
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let readiness = wendao_graph_link_evidence_readiness_evidence(
        facts.runtime_stats.warmup,
        facts.runtime_stats.benchmark,
        facts.max_in_flight,
        facts.runtime_stats.active_in_flight,
        facts.runtime_stats.queue_depth,
    )
    .with_fallback_available(facts.fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
}

/// Returns runtime stats derived from a `WendaoGraph.jl` `LinkGraph` host probe report.
#[must_use]
pub fn wendao_graph_link_evidence_runtime_stats_from_host_probe(
    report: &WendaoGraphLinkGraphHostProbeReport,
) -> JuliaRuntimeStats {
    JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::NotRequired)
        .with_latency_ms(
            Some(latency_ms_as_u32(report.warm_median_ms)),
            Some(latency_ms_as_u32(report.warm_p95_ms)),
        )
}

/// Returns runtime stats derived from a full-structural `WendaoGraph.jl`
/// `LinkGraph` host probe report.
#[must_use]
pub fn wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
) -> JuliaRuntimeStats {
    wendao_graph_link_evidence_runtime_stats_from_host_probe(&report.base)
}

/// Returns readiness evidence derived from a full-structural `WendaoGraph.jl`
/// `LinkGraph` host probe report.
#[must_use]
pub fn wendao_graph_link_evidence_readiness_evidence_from_full_structural_host_probe(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    let benchmark = if report.base.sample_count > 0 {
        BenchmarkState::NotRequired
    } else {
        BenchmarkState::Failed
    };
    wendao_graph_link_evidence_readiness_evidence(
        WarmupState::Ready,
        benchmark,
        max_in_flight,
        active_in_flight,
        queue_depth,
    )
}

/// Returns readiness evidence for the `WendaoGraph.jl` `PageIndex` reasoning profile.
#[must_use]
pub fn wendao_graph_page_index_reasoning_readiness_evidence(
    warmup: WarmupState,
    benchmark: BenchmarkState,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    julia_static_contract_readiness_evidence(
        JuliaStaticContractReadinessProfile {
            capability: LaneCapability::GraphEvidenceCompute,
            profile_id: WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
            schema_version: WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
        },
        warmup,
        benchmark,
        JuliaReadinessWindow {
            max_in_flight,
            active_in_flight,
            queue_depth,
        },
    )
}

/// Returns a schedule plan for the `WendaoGraph.jl` `PageIndex` reasoning profile.
#[must_use]
pub fn wendao_graph_page_index_reasoning_schedule_plan(
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let readiness = wendao_graph_page_index_reasoning_readiness_evidence(
        facts.runtime_stats.warmup,
        facts.runtime_stats.benchmark,
        facts.max_in_flight,
        facts.runtime_stats.active_in_flight,
        facts.runtime_stats.queue_depth,
    )
    .with_fallback_available(facts.fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
}

/// Returns runtime stats derived from a `WendaoGraph.jl` `PageIndex` host probe report.
#[must_use]
pub fn wendao_graph_page_index_reasoning_runtime_stats_from_host_probe(
    report: &WendaoGraphPageIndexHostProbeReport,
) -> JuliaRuntimeStats {
    JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::NotRequired)
        .with_latency_ms(
            Some(latency_ms_as_u32(report.warm_median_ms)),
            Some(latency_ms_as_u32(report.warm_p95_ms)),
        )
}

/// Returns runtime stats derived from a `WendaoGraph.jl` `PageIndex`
/// planner-action host probe report.
#[must_use]
pub fn wendao_graph_page_index_reasoning_runtime_stats_from_planner_action_host_probe(
    report: &WendaoGraphPageIndexPlannerActionHostProbeReport,
) -> JuliaRuntimeStats {
    wendao_graph_page_index_reasoning_runtime_stats_from_host_probe(&report.base)
}

/// Returns readiness evidence derived from a `WendaoGraph.jl` `PageIndex` host
/// probe report.
#[must_use]
pub fn wendao_graph_page_index_reasoning_readiness_evidence_from_host_probe(
    report: &WendaoGraphPageIndexHostProbeReport,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    let benchmark = if report.sample_count > 0 {
        BenchmarkState::NotRequired
    } else {
        BenchmarkState::Failed
    };
    wendao_graph_page_index_reasoning_readiness_evidence(
        WarmupState::Ready,
        benchmark,
        max_in_flight,
        active_in_flight,
        queue_depth,
    )
}

/// Returns readiness evidence for the `WendaoGraph.jl` GNN reasoning profile.
#[must_use]
pub fn wendao_graph_gnn_reasoning_readiness_evidence(
    warmup: WarmupState,
    benchmark: BenchmarkState,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    julia_static_contract_readiness_evidence(
        JuliaStaticContractReadinessProfile {
            capability: LaneCapability::GraphEvidenceCompute,
            profile_id: WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
            schema_version: WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
        },
        warmup,
        benchmark,
        JuliaReadinessWindow {
            max_in_flight,
            active_in_flight,
            queue_depth,
        },
    )
}

/// Returns runtime stats derived from a `WendaoGraph.jl` GNN host probe report.
#[must_use]
pub fn wendao_graph_gnn_runtime_stats_from_host_probe(
    report: &WendaoGraphGnnHostProbeReport,
) -> JuliaRuntimeStats {
    JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::NotRequired)
        .with_latency_ms(
            Some(latency_ms_as_u32(report.warm_median_ms)),
            Some(latency_ms_as_u32(report.warm_p95_ms)),
        )
}

/// Returns accelerator diagnostics derived from a `WendaoGraph.jl` GNN host
/// probe report.
#[must_use]
pub fn wendao_graph_gnn_accelerator_diagnostics_from_host_probe(
    report: &WendaoGraphGnnHostProbeReport,
) -> Vec<JuliaAcceleratorDiagnostics> {
    vec![
        JuliaAcceleratorDiagnostics::new(
            "metal",
            report.backend_load.metal_loaded,
            report.metal_functional,
        )
        .with_observed_output_count(
            (report.metal_score_count > 0)
                .then_some(saturating_usize_to_u32(report.metal_score_count)),
        ),
        JuliaAcceleratorDiagnostics::new("cuda", report.backend_load.cuda_loaded, false),
        JuliaAcceleratorDiagnostics::new("amdgpu", report.backend_load.amdgpu_loaded, false),
    ]
}

/// Returns readiness evidence derived from a `WendaoGraph.jl` GNN host probe
/// report.
#[must_use]
pub fn wendao_graph_gnn_readiness_evidence_from_host_probe(
    report: &WendaoGraphGnnHostProbeReport,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    wendao_graph_gnn_reasoning_readiness_evidence(
        WarmupState::Ready,
        BenchmarkState::NotRequired,
        max_in_flight,
        active_in_flight,
        queue_depth,
    )
    .with_accelerator_diagnostics(wendao_graph_gnn_accelerator_diagnostics_from_host_probe(
        report,
    ))
}

/// Returns a schedule plan for the `WendaoGraph.jl` GNN reasoning profile.
#[must_use]
pub fn wendao_graph_gnn_reasoning_schedule_plan(
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let readiness = wendao_graph_gnn_reasoning_readiness_evidence(
        facts.runtime_stats.warmup,
        facts.runtime_stats.benchmark,
        facts.max_in_flight,
        facts.runtime_stats.active_in_flight,
        facts.runtime_stats.queue_depth,
    )
    .with_fallback_available(facts.fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
}

/// Returns a schedule plan for one staged `WendaoGraph.jl` algorithm id.
///
/// Unknown algorithm ids return `None`; the caller can then choose an
/// owner-specific fallback or skip Julia for that request.
#[must_use]
pub fn wendaograph_algorithm_schedule_plan(
    algorithm_id: &str,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<JuliaSchedulePlan> {
    let reference = wendaograph_algorithm_ref(algorithm_id)?;
    let shape = reference.task_shape(workload);
    match reference.profile_id {
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID => {
            Some(wendao_graph_link_evidence_schedule_plan(shape, facts))
        }
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID => Some(
            wendao_graph_page_index_reasoning_schedule_plan(shape, facts),
        ),
        WENDAO_GRAPH_GNN_REASONING_PROFILE_ID => {
            Some(wendao_graph_gnn_reasoning_schedule_plan(shape, facts))
        }
        _ => None,
    }
}

/// Returns a schedule plan for one reasoning-tree backend frontier evidence
/// kind.
///
/// Evidence kinds that remain Rust-owned, such as authority and negative-guard
/// checks, return `None`.
#[must_use]
pub fn wendaograph_frontier_schedule_plan(
    evidence_kind: &str,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<JuliaSchedulePlan> {
    let reference = wendaograph_frontier_algorithm_ref(evidence_kind)?;
    wendaograph_algorithm_schedule_plan(reference.algorithm_id, workload, facts)
}

/// Projects every relationship-search algorithm into host-probe-backed
/// scheduling evidence.
#[must_use]
pub fn wendaograph_relationship_search_evidence_from_full_structural_host_probe(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Vec<WendaoGraphRelationshipSearchEvidence> {
    wendaograph_relationship_search_algorithm_refs()
        .iter()
        .filter_map(|reference| {
            wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe(
                reference.algorithm_id,
                report,
                workload,
                facts,
            )
        })
        .collect()
}

/// Projects one relationship-search algorithm id into host-probe-backed
/// scheduling evidence.
///
/// Unknown ids, non-relationship-search ids, or ids that cannot route through
/// the existing algorithm schedule helper return `None`.
#[must_use]
pub fn wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe(
    algorithm_id: &str,
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<WendaoGraphRelationshipSearchEvidence> {
    let algorithm = wendaograph_algorithm_ref(algorithm_id)?;
    if algorithm.family != "relationship_search" {
        return None;
    }

    let runtime_stats =
        relationship_search_runtime_stats_from_full_structural_host_probe(report, facts);
    let facts = JuliaProfileSchedulingFacts {
        runtime_stats,
        ..facts
    };
    let schedule_plan = wendaograph_algorithm_schedule_plan(algorithm_id, workload, facts)?;
    let (probe_table, probe_rows) = relationship_search_probe_rows(report, algorithm_id);
    Some(WendaoGraphRelationshipSearchEvidence {
        algorithm,
        probe_table,
        probe_rows,
        runtime_stats,
        schedule_plan,
    })
}

/// Returns readiness evidence for one `WendaoSearch.jl` graph-structural profile.
#[must_use]
pub fn wendaosearch_graph_structural_readiness_evidence(
    route_kind: GraphStructuralRouteKind,
    warmup: WarmupState,
    benchmark: BenchmarkState,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    julia_static_contract_readiness_evidence(
        JuliaStaticContractReadinessProfile {
            capability: LaneCapability::GraphSearchCompute,
            profile_id: wendaosearch_graph_structural_profile_id(route_kind),
            schema_version: JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION,
        },
        warmup,
        benchmark,
        JuliaReadinessWindow {
            max_in_flight,
            active_in_flight,
            queue_depth,
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
        route_kind,
        facts.runtime_stats.warmup,
        facts.runtime_stats.benchmark,
        facts.max_in_flight,
        facts.runtime_stats.active_in_flight,
        facts.runtime_stats.queue_depth,
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
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
    warmup: WarmupState,
    benchmark: BenchmarkState,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
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
    let readiness = wendaosearch_legacy_rerank_readiness_evidence(
        runtime,
        facts.runtime_stats.warmup,
        facts.runtime_stats.benchmark,
        facts.runtime_stats.active_in_flight,
        facts.runtime_stats.queue_depth,
    )
    .with_admission_window(
        facts.max_in_flight,
        facts.runtime_stats.active_in_flight,
        facts.runtime_stats.queue_depth,
    )
    .with_fallback_available(facts.fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
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
    let readiness = memory_julia_compute_readiness_evidence(
        runtime,
        profile,
        facts.runtime_stats.warmup,
        facts.runtime_stats.benchmark,
        facts.runtime_stats.active_in_flight,
        facts.runtime_stats.queue_depth,
    )
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

const fn wendaosearch_graph_structural_profile_id(
    route_kind: GraphStructuralRouteKind,
) -> &'static str {
    match route_kind {
        GraphStructuralRouteKind::StructuralRerank => WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
        GraphStructuralRouteKind::ConstraintFilter => WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    }
}

fn julia_static_contract_readiness_evidence(
    profile: JuliaStaticContractReadinessProfile,
    warmup: WarmupState,
    benchmark: BenchmarkState,
    window: JuliaReadinessWindow,
) -> JuliaReadinessEvidence {
    JuliaReadinessEvidence::new(profile.capability, profile.profile_id)
        .with_schema_version(profile.schema_version)
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(warmup)
        .with_benchmark(benchmark)
        .with_admission_window(
            window.max_in_flight,
            window.active_in_flight,
            window.queue_depth,
        )
        .with_fallback_available(false)
}

fn julia_schedule_plan_from_readiness(
    readiness: JuliaReadinessEvidence,
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    JuliaSchedulingInput::new(readiness, shape, facts.runtime_stats)
        .with_fallback_available(facts.fallback_available)
        .with_deadline_ms(facts.deadline_ms)
        .with_target_latency_ms(facts.target_latency_ms)
        .plan()
}

#[derive(Clone, Copy)]
struct JuliaStaticContractReadinessProfile {
    capability: LaneCapability,
    profile_id: &'static str,
    schema_version: &'static str,
}

#[derive(Clone, Copy)]
struct JuliaReadinessWindow {
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
}

fn max_in_flight_as_u32(max_in_flight_requests: u64) -> u32 {
    u32::try_from(max_in_flight_requests).unwrap_or(u32::MAX)
}

fn saturating_usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn latency_ms_as_u32(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    if value < 1.0 {
        return 1;
    }
    if value >= 4_294_967_295.0 {
        return u32::MAX;
    }
    format!("{value:.0}").parse().unwrap_or(u32::MAX)
}

fn relationship_search_runtime_stats_from_full_structural_host_probe(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaRuntimeStats {
    let probe_stats =
        wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe(report)
            .with_error_rate_basis_points(facts.runtime_stats.error_rate_basis_points)
            .with_queue(
                facts.runtime_stats.queue_depth,
                facts.runtime_stats.active_in_flight,
            );
    let benchmark = match facts.runtime_stats.benchmark {
        BenchmarkState::Unknown => probe_stats.benchmark,
        benchmark => benchmark,
    };
    probe_stats.with_benchmark(benchmark)
}

fn relationship_search_probe_rows(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    algorithm_id: &str,
) -> (Option<&'static str>, Option<u32>) {
    match algorithm_id {
        "relationship_search.hnsw_semantic_fanout"
        | "relationship_search.semantic_overlay_edges" => (
            Some("semantic_overlay"),
            Some(saturating_usize_to_u32(report.base.semantic_overlay_rows)),
        ),
        "relationship_search.moc_community_grouping" => (
            Some("topology_communities"),
            Some(saturating_usize_to_u32(report.topology_community_rows)),
        ),
        "relationship_search.community_bridge_links" => (
            Some("topology_community_links"),
            Some(saturating_usize_to_u32(report.topology_community_link_rows)),
        ),
        "relationship_search.community_frontier_ranking" => (
            Some("topology_community_frontier"),
            Some(saturating_usize_to_u32(
                report.topology_community_frontier_rows,
            )),
        ),
        "relationship_search.ppr_like_relatedness" => (
            Some("diffusion_scores"),
            Some(saturating_usize_to_u32(report.base.diffusion_rows)),
        ),
        "relationship_search.graph_search_ranking" => (
            Some("link_frontier"),
            Some(saturating_usize_to_u32(report.base.frontier_rows)),
        ),
        "relationship_search.topology_candidate_ranking" => (
            Some("topology_candidates"),
            Some(saturating_usize_to_u32(report.base.topology_candidate_rows)),
        ),
        "relationship_search.large_object_graph_traversal" => (
            Some("components"),
            Some(saturating_usize_to_u32(report.component_rows)),
        ),
        "relationship_search.graph_snapshot_traversal" => (
            Some("graph_metrics"),
            Some(saturating_usize_to_u32(report.base.graph_metric_rows)),
        ),
        _ => (None, None),
    }
}

#[cfg(test)]
#[path = "../tests/unit/polyglot.rs"]
mod tests;
