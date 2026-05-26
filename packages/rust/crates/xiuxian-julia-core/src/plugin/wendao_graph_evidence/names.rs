//! Canonical `WendaoGraph` evidence route and table names.

/// Default schema version for the Rust mirror of the `WendaoGraph` evidence contract.
pub const WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION: &str = "v0-draft";

/// Planned Flight route for `LinkGraph` evidence requests handled by `WendaoGraph.jl`.
pub const WENDAO_GRAPH_LINK_EVIDENCE_ROUTE: &str = "/graph/link/evidence";

/// Canonical `WendaoGraph` evidence request table names.
pub const WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_NAMES: [&str; 5] = [
    "nodes",
    "edges",
    "seeds",
    "semantic_neighbors",
    "semantic_overlay",
];

/// Canonical `WendaoGraph` evidence response table names.
pub const WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES: [&str; 17] = [
    "graph_metrics",
    "components",
    "topology_profile",
    "topology_candidates",
    "topology_bottlenecks",
    "topology_communities",
    "topology_cover",
    "topology_core",
    "topology_boundary",
    "topology_transitions",
    "topology_gateways",
    "topology_community_summaries",
    "topology_community_links",
    "topology_community_frontier",
    "semantic_overlay",
    "diffusion_scores",
    "link_frontier",
];

/// Canonical `WendaoGraph` `PageIndex` reasoning request table names.
pub const WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_NAMES: [&str; 3] =
    ["page_index_nodes", "page_index_edges", "page_index_seeds"];

/// Canonical `WendaoGraph` `PageIndex` reasoning response table names.
pub const WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_NAMES: [&str; 6] = [
    "page_index_nodes",
    "page_index_edges",
    "page_index_seeds",
    "reasoning_frontier",
    "disclosure_trace",
    "page_index_planner_actions",
];
