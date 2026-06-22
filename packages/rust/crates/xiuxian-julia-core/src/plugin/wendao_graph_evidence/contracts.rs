//! `WendaoGraph` evidence table contract manifests.

use super::page_index_columns::{
    PAGE_INDEX_DISCLOSURE_TRACE_COLUMNS, PAGE_INDEX_PLANNER_ACTION_COLUMNS,
    PAGE_INDEX_REASONING_EDGE_COLUMNS, PAGE_INDEX_REASONING_FRONTIER_COLUMNS,
    PAGE_INDEX_REASONING_NODE_COLUMNS, PAGE_INDEX_REASONING_SEED_COLUMNS,
};
use super::request_columns::{
    EDGE_COLUMNS, NODE_COLUMNS, SEED_COLUMNS, SEMANTIC_NEIGHBOR_COLUMNS, SEMANTIC_OVERLAY_COLUMNS,
};
use super::response_columns::{
    COMPONENT_COLUMNS, DIFFUSION_SCORE_COLUMNS, GRAPH_METRIC_COLUMNS, LINK_FRONTIER_COLUMNS,
    TOPOLOGY_BOTTLENECK_COLUMNS, TOPOLOGY_BOUNDARY_COLUMNS, TOPOLOGY_CANDIDATE_COLUMNS,
    TOPOLOGY_COMMUNITY_COLUMNS, TOPOLOGY_COMMUNITY_FRONTIER_COLUMNS,
    TOPOLOGY_COMMUNITY_LINK_COLUMNS, TOPOLOGY_COMMUNITY_SUMMARY_COLUMNS, TOPOLOGY_CORE_COLUMNS,
    TOPOLOGY_COVER_COLUMNS, TOPOLOGY_GATEWAY_COLUMNS, TOPOLOGY_PROFILE_COLUMNS,
    TOPOLOGY_TRANSITION_COLUMNS,
};
use super::types::{
    WendaoGraphEvidenceColumnContract, WendaoGraphEvidenceTableContract,
    WendaoGraphEvidenceTableKind,
};

/// Canonical `WendaoGraph` request table contracts.
pub const WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_CONTRACTS: [WendaoGraphEvidenceTableContract; 5] = [
    request_table("nodes", true, &NODE_COLUMNS),
    request_table("edges", true, &EDGE_COLUMNS),
    request_table("seeds", false, &SEED_COLUMNS),
    request_table("semantic_neighbors", false, &SEMANTIC_NEIGHBOR_COLUMNS),
    request_table("semantic_overlay", false, &SEMANTIC_OVERLAY_COLUMNS),
];

/// Canonical `WendaoGraph` response table contracts.
pub const WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_CONTRACTS: [WendaoGraphEvidenceTableContract; 17] = [
    response_table("graph_metrics", &GRAPH_METRIC_COLUMNS),
    response_table("components", &COMPONENT_COLUMNS),
    response_table("topology_profile", &TOPOLOGY_PROFILE_COLUMNS),
    response_table("topology_candidates", &TOPOLOGY_CANDIDATE_COLUMNS),
    response_table("topology_bottlenecks", &TOPOLOGY_BOTTLENECK_COLUMNS),
    response_table("topology_communities", &TOPOLOGY_COMMUNITY_COLUMNS),
    response_table("topology_cover", &TOPOLOGY_COVER_COLUMNS),
    response_table("topology_core", &TOPOLOGY_CORE_COLUMNS),
    response_table("topology_boundary", &TOPOLOGY_BOUNDARY_COLUMNS),
    response_table("topology_transitions", &TOPOLOGY_TRANSITION_COLUMNS),
    response_table("topology_gateways", &TOPOLOGY_GATEWAY_COLUMNS),
    response_table(
        "topology_community_summaries",
        &TOPOLOGY_COMMUNITY_SUMMARY_COLUMNS,
    ),
    response_table("topology_community_links", &TOPOLOGY_COMMUNITY_LINK_COLUMNS),
    response_table(
        "topology_community_frontier",
        &TOPOLOGY_COMMUNITY_FRONTIER_COLUMNS,
    ),
    response_table("semantic_overlay", &SEMANTIC_OVERLAY_COLUMNS),
    response_table("diffusion_scores", &DIFFUSION_SCORE_COLUMNS),
    response_table("link_frontier", &LINK_FRONTIER_COLUMNS),
];

/// Canonical `WendaoGraph` `PageIndex` reasoning request table contracts.
pub const WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_CONTRACTS:
    [WendaoGraphEvidenceTableContract; 3] = [
    request_table("page_index_nodes", true, &PAGE_INDEX_REASONING_NODE_COLUMNS),
    request_table("page_index_edges", true, &PAGE_INDEX_REASONING_EDGE_COLUMNS),
    request_table(
        "page_index_seeds",
        false,
        &PAGE_INDEX_REASONING_SEED_COLUMNS,
    ),
];

/// Canonical `WendaoGraph` `PageIndex` reasoning response table contracts.
pub const WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_CONTRACTS:
    [WendaoGraphEvidenceTableContract; 6] = [
    response_table("page_index_nodes", &PAGE_INDEX_REASONING_NODE_COLUMNS),
    response_table("page_index_edges", &PAGE_INDEX_REASONING_EDGE_COLUMNS),
    response_table("page_index_seeds", &PAGE_INDEX_REASONING_SEED_COLUMNS),
    response_table("reasoning_frontier", &PAGE_INDEX_REASONING_FRONTIER_COLUMNS),
    response_table("disclosure_trace", &PAGE_INDEX_DISCLOSURE_TRACE_COLUMNS),
    response_table(
        "page_index_planner_actions",
        &PAGE_INDEX_PLANNER_ACTION_COLUMNS,
    ),
];

const fn request_table(
    table_name: &'static str,
    required: bool,
    columns: &'static [WendaoGraphEvidenceColumnContract],
) -> WendaoGraphEvidenceTableContract {
    WendaoGraphEvidenceTableContract {
        table_name,
        kind: WendaoGraphEvidenceTableKind::Request,
        required,
        columns,
    }
}

const fn response_table(
    table_name: &'static str,
    columns: &'static [WendaoGraphEvidenceColumnContract],
) -> WendaoGraphEvidenceTableContract {
    WendaoGraphEvidenceTableContract {
        table_name,
        kind: WendaoGraphEvidenceTableKind::Response,
        required: true,
        columns,
    }
}
