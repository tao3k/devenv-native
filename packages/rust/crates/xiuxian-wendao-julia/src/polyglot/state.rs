//! Profile refs and scheduling facts for polyglot Julia contracts.

use super::wendaograph_algorithms::WendaoGraphAlgorithmRef;
use xiuxian_polyglot_orchestrator::{
    JuliaRuntimeStats, JuliaSchedulePlan, PolyglotControlSnapshot, RouteProfileRef,
    SnapshotInvariantError,
};
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

use crate::compatibility::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, LinkGraphJuliaRerankRuntimeConfig,
};
use crate::memory::{
    MemoryJuliaComputeManifestRow, MemoryJuliaComputeProfile,
    build_memory_julia_compute_manifest_rows,
};
use crate::{
    GraphStructuralRouteKind, WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
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

pub(super) const fn wendaosearch_graph_structural_profile_id(
    route_kind: GraphStructuralRouteKind,
) -> &'static str {
    match route_kind {
        GraphStructuralRouteKind::StructuralRerank => WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
        GraphStructuralRouteKind::ConstraintFilter => WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    }
}
