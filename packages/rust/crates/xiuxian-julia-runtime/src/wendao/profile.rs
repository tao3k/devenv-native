//! Stable Wendao-facing Julia profile identities.

/// Stable profile id for the `WendaoGraph.jl` link-evidence contract.
pub const WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID: &str = "wendao_graph_link_evidence";
/// Flight route for the `WendaoGraph.jl` link-evidence contract.
pub const WENDAO_GRAPH_LINK_EVIDENCE_ROUTE: &str = "/graph/link/evidence";
/// Contract version for `WendaoGraph.jl` graph-evidence routes.
pub const WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION: &str = "v0-draft";
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
/// Default Flight route for the legacy `WendaoSearch.jl` rerank route.
pub const WENDAOSEARCH_LEGACY_RERANK_ROUTE: &str = "/rerank";
/// Stable profile id for the `WendaoSearch.jl` structural-rerank route.
pub const WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID: &str = "wendaosearch_structural_rerank";
/// Flight route for the `WendaoSearch.jl` structural-rerank route.
pub const WENDAOSEARCH_STRUCTURAL_RERANK_ROUTE: &str = "/graph/structural/rerank";
/// Stable profile id for the `WendaoSearch.jl` constraint-filter route.
pub const WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID: &str = "wendaosearch_constraint_filter";
/// Flight route for the `WendaoSearch.jl` constraint-filter route.
pub const WENDAOSEARCH_CONSTRAINT_FILTER_ROUTE: &str = "/graph/structural/filter";
/// Contract version for `WendaoSearch.jl` graph-structural routes.
pub const WENDAOSEARCH_GRAPH_STRUCTURAL_SCHEMA_VERSION: &str = "v0-draft";

/// Stable Rust-facing `WendaoGraph.jl` algorithm identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WendaoGraphAlgorithmId(pub &'static str);

/// Stable Rust-facing `WendaoGraph.jl` profile identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WendaoGraphProfileId(pub &'static str);
