//! `WendaoGraph` evidence response table columns.

use super::types::{WendaoGraphEvidenceColumnContract, WendaoGraphEvidenceColumnType, column};

pub(super) const GRAPH_METRIC_COLUMNS: [WendaoGraphEvidenceColumnContract; 5] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("in_degree", WendaoGraphEvidenceColumnType::Int64),
    column("out_degree", WendaoGraphEvidenceColumnType::Int64),
    column("degree", WendaoGraphEvidenceColumnType::Int64),
];
pub(super) const COMPONENT_COLUMNS: [WendaoGraphEvidenceColumnContract; 5] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("component_id", WendaoGraphEvidenceColumnType::Int64),
    column("component_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("component_size", WendaoGraphEvidenceColumnType::Int64),
];
pub(super) const TOPOLOGY_PROFILE_COLUMNS: [WendaoGraphEvidenceColumnContract; 11] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("weak_component_id", WendaoGraphEvidenceColumnType::Int64),
    column("weak_component_size", WendaoGraphEvidenceColumnType::Int64),
    column("strong_component_id", WendaoGraphEvidenceColumnType::Int64),
    column(
        "strong_component_size",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("pagerank_score", WendaoGraphEvidenceColumnType::Float64),
    column("degree_centrality", WendaoGraphEvidenceColumnType::Float64),
    column("topology_prior", WendaoGraphEvidenceColumnType::Float64),
    column("topology_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_CANDIDATE_COLUMNS: [WendaoGraphEvidenceColumnContract; 9] = [
    column("seed_id", WendaoGraphEvidenceColumnType::Utf8),
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("distance", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("topology_score", WendaoGraphEvidenceColumnType::Float64),
    column("topology_prior", WendaoGraphEvidenceColumnType::Float64),
    column("topology_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_BOTTLENECK_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("is_articulation", WendaoGraphEvidenceColumnType::Boolean),
    column(
        "bridge_endpoint_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "biconnected_component_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("bottleneck_score", WendaoGraphEvidenceColumnType::Float64),
    column("bottleneck_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_COMMUNITY_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column("community_size", WendaoGraphEvidenceColumnType::Int64),
    column("community_count", WendaoGraphEvidenceColumnType::Int64),
    column("community_score", WendaoGraphEvidenceColumnType::Float64),
    column("modularity_score", WendaoGraphEvidenceColumnType::Float64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_COVER_COLUMNS: [WendaoGraphEvidenceColumnContract; 9] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("anchor_id", WendaoGraphEvidenceColumnType::Utf8),
    column("anchor_vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("is_anchor", WendaoGraphEvidenceColumnType::Boolean),
    column("cover_distance", WendaoGraphEvidenceColumnType::Int64),
    column("anchor_degree", WendaoGraphEvidenceColumnType::Int64),
    column("cover_score", WendaoGraphEvidenceColumnType::Float64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_CORE_COLUMNS: [WendaoGraphEvidenceColumnContract; 7] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("core_number", WendaoGraphEvidenceColumnType::Int64),
    column("max_core_number", WendaoGraphEvidenceColumnType::Int64),
    column("core_score", WendaoGraphEvidenceColumnType::Float64),
    column("core_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_BOUNDARY_COLUMNS: [WendaoGraphEvidenceColumnContract; 9] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column(
        "internal_neighbor_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "external_neighbor_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("boundary_ratio", WendaoGraphEvidenceColumnType::Float64),
    column("boundary_score", WendaoGraphEvidenceColumnType::Float64),
    column("boundary_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_TRANSITION_COLUMNS: [WendaoGraphEvidenceColumnContract; 10] = [
    column("source_id", WendaoGraphEvidenceColumnType::Utf8),
    column("target_id", WendaoGraphEvidenceColumnType::Utf8),
    column("source_index", WendaoGraphEvidenceColumnType::Int64),
    column("target_index", WendaoGraphEvidenceColumnType::Int64),
    column("source_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("target_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("is_cross_community", WendaoGraphEvidenceColumnType::Boolean),
    column("transition_score", WendaoGraphEvidenceColumnType::Float64),
    column("transition_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_GATEWAY_COLUMNS: [WendaoGraphEvidenceColumnContract; 9] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column(
        "incoming_transition_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "outgoing_transition_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "transition_community_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("gateway_score", WendaoGraphEvidenceColumnType::Float64),
    column("gateway_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_COMMUNITY_SUMMARY_COLUMNS: [WendaoGraphEvidenceColumnContract; 11] = [
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column("community_size", WendaoGraphEvidenceColumnType::Int64),
    column("community_count", WendaoGraphEvidenceColumnType::Int64),
    column(
        "representative_node_id",
        WendaoGraphEvidenceColumnType::Utf8,
    ),
    column(
        "representative_vertex_index",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("gateway_count", WendaoGraphEvidenceColumnType::Int64),
    column("boundary_count", WendaoGraphEvidenceColumnType::Int64),
    column("transition_count", WendaoGraphEvidenceColumnType::Int64),
    column("summary_score", WendaoGraphEvidenceColumnType::Float64),
    column("summary_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_COMMUNITY_LINK_COLUMNS: [WendaoGraphEvidenceColumnContract; 10] = [
    column("source_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("target_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("transition_count", WendaoGraphEvidenceColumnType::Int64),
    column(
        "representative_source_id",
        WendaoGraphEvidenceColumnType::Utf8,
    ),
    column(
        "representative_target_id",
        WendaoGraphEvidenceColumnType::Utf8,
    ),
    column(
        "representative_source_index",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "representative_target_index",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "representative_transition_score",
        WendaoGraphEvidenceColumnType::Float64,
    ),
    column("link_score", WendaoGraphEvidenceColumnType::Float64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const TOPOLOGY_COMMUNITY_FRONTIER_COLUMNS: [WendaoGraphEvidenceColumnContract; 15] = [
    column("tree_id", WendaoGraphEvidenceColumnType::Utf8),
    column("parent_step_id", WendaoGraphEvidenceColumnType::Utf8),
    column("step_id", WendaoGraphEvidenceColumnType::Utf8),
    column("parent_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column("depth", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column(
        "representative_node_id",
        WendaoGraphEvidenceColumnType::Utf8,
    ),
    column(
        "representative_vertex_index",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("community_score", WendaoGraphEvidenceColumnType::Float64),
    column("link_score", WendaoGraphEvidenceColumnType::Float64),
    column("path_score", WendaoGraphEvidenceColumnType::Float64),
    column("transition_count", WendaoGraphEvidenceColumnType::Int64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("disclosure_budget", WendaoGraphEvidenceColumnType::Int64),
];
pub(super) const DIFFUSION_SCORE_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("diffusion_score", WendaoGraphEvidenceColumnType::Float64),
    column("seed_score", WendaoGraphEvidenceColumnType::Float64),
    column("link_score", WendaoGraphEvidenceColumnType::Float64),
    column("semantic_score", WendaoGraphEvidenceColumnType::Float64),
    column("iteration_count", WendaoGraphEvidenceColumnType::Int64),
    column("residual", WendaoGraphEvidenceColumnType::Float64),
];
pub(super) const LINK_FRONTIER_COLUMNS: [WendaoGraphEvidenceColumnContract; 10] = [
    column("tree_id", WendaoGraphEvidenceColumnType::Utf8),
    column("parent_id", WendaoGraphEvidenceColumnType::Utf8),
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("depth", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("diffusion_score", WendaoGraphEvidenceColumnType::Float64),
    column("path_score", WendaoGraphEvidenceColumnType::Float64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("disclosure_budget", WendaoGraphEvidenceColumnType::Int64),
];
