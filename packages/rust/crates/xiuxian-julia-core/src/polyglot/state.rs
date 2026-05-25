//! Profile refs and scheduling facts for polyglot Julia contracts.

use xiuxian_polyglot_orchestrator::{
    PolyglotControlSnapshot, RouteProfileRef, SnapshotInvariantError,
    WendaoSearchLegacyRerankProfileRefInput,
};
pub use xiuxian_polyglot_orchestrator::{
    WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
    WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
    memory_julia_compute_profile_ref, memory_julia_compute_profile_refs,
    wendao_graph_gnn_reasoning_profile_ref, wendao_graph_link_evidence_profile_ref,
    wendao_graph_page_index_reasoning_profile_ref,
};

use crate::GraphStructuralRouteKind;
use crate::compatibility::link_graph::LinkGraphJuliaRerankRuntimeConfig;
use crate::memory::MemoryJuliaComputeManifestRow;

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

/// Returns a typed ref for one `WendaoSearch.jl` graph-structural route.
#[must_use]
pub fn wendaosearch_graph_structural_profile_ref(
    route_kind: GraphStructuralRouteKind,
) -> RouteProfileRef {
    match route_kind {
        GraphStructuralRouteKind::StructuralRerank => {
            xiuxian_polyglot_orchestrator::wendaosearch_structural_rerank_profile_ref()
        }
        GraphStructuralRouteKind::ConstraintFilter => {
            xiuxian_polyglot_orchestrator::wendaosearch_constraint_filter_profile_ref()
        }
    }
}

/// Returns typed refs for the staged `WendaoSearch.jl` graph-structural routes.
#[must_use]
pub fn wendaosearch_graph_structural_profile_refs() -> Vec<RouteProfileRef> {
    xiuxian_polyglot_orchestrator::wendaosearch_graph_structural_profile_refs()
}

/// Returns the typed ref for the legacy `WendaoSearch.jl` rerank route.
#[must_use]
pub fn wendaosearch_legacy_rerank_profile_ref(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> RouteProfileRef {
    xiuxian_polyglot_orchestrator::wendaosearch_legacy_rerank_profile_ref(
        WendaoSearchLegacyRerankProfileRefInput {
            route: runtime.route.as_deref(),
            schema_version: runtime.schema_version.as_deref(),
        },
    )
}

/// Returns graph-family Julia route refs currently known to the Rust scheduler.
#[must_use]
pub fn julia_graph_compute_profile_refs(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> Vec<RouteProfileRef> {
    xiuxian_polyglot_orchestrator::julia_graph_compute_profile_refs(
        WendaoSearchLegacyRerankProfileRefInput {
            route: runtime.route.as_deref(),
            schema_version: runtime.schema_version.as_deref(),
        },
    )
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

pub(super) const fn wendaosearch_graph_structural_profile_id(
    route_kind: GraphStructuralRouteKind,
) -> &'static str {
    match route_kind {
        GraphStructuralRouteKind::StructuralRerank => WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
        GraphStructuralRouteKind::ConstraintFilter => WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    }
}
