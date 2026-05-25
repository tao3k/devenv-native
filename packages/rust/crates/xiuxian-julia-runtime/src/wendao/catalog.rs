//! Stable `WendaoGraph.jl` algorithm catalog facts.

use super::{
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WendaoGraphAlgorithmId, WendaoGraphProfileId,
};

/// Runtime-neutral complexity hint for one Julia algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WendaoGraphAlgorithmComplexity {
    /// Lightweight row or metadata operation.
    Simple,
    /// Moderate graph or table operation.
    Balanced,
    /// Structurally heavy graph operation.
    Heavy,
}

/// Runtime-owned catalog entry for one `WendaoGraph.jl` algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WendaoGraphAlgorithmRef {
    /// Stable Rust-facing algorithm id.
    pub algorithm_id: &'static str,
    /// Coarse algorithm family.
    pub family: &'static str,
    /// Julia profile that owns this algorithm.
    pub profile_id: &'static str,
    /// Julia function or host entrypoint that owns the implementation.
    pub julia_entrypoint: &'static str,
    /// Output table produced by the algorithm when it has a table surface.
    pub output_table: Option<&'static str>,
    /// Runtime-neutral complexity hint.
    pub complexity: WendaoGraphAlgorithmComplexity,
}

impl WendaoGraphAlgorithmRef {
    /// Creates an inert `WendaoGraph.jl` algorithm catalog entry.
    #[must_use]
    const fn new(
        algorithm_id: &'static str,
        family: &'static str,
        profile_id: &'static str,
        julia_entrypoint: &'static str,
        output_table: Option<&'static str>,
        complexity: WendaoGraphAlgorithmComplexity,
    ) -> Self {
        Self {
            algorithm_id,
            family,
            profile_id,
            julia_entrypoint,
            output_table,
            complexity,
        }
    }

    /// Returns whether this algorithm is marked as structurally heavy.
    #[must_use]
    pub const fn is_heavy(self) -> bool {
        matches!(self.complexity, WendaoGraphAlgorithmComplexity::Heavy)
    }
}

const WENDAO_GRAPH_LINK_GRAPH_ALGORITHMS: &[WendaoGraphAlgorithmRef] = &[
    link_graph_algorithm(
        "link_graph.graph_metrics",
        "WendaoGraph.graph_metric_rows",
        Some("graph_metrics"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.components",
        "WendaoGraph.component_rows",
        Some("components"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.topology_profile",
        "WendaoGraph.topology_profile_rows",
        Some("topology_profile"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_candidates",
        "WendaoGraph.topology_candidate_rows",
        Some("topology_candidates"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_bottlenecks",
        "WendaoGraph.topology_bottleneck_rows",
        Some("topology_bottlenecks"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_communities",
        "WendaoGraph.topology_community_rows",
        Some("topology_communities"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_cover",
        "WendaoGraph.topology_cover_rows",
        Some("topology_cover"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.topology_core",
        "WendaoGraph.topology_core_rows",
        Some("topology_core"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_boundary",
        "WendaoGraph.topology_boundary_rows",
        Some("topology_boundary"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_transitions",
        "WendaoGraph.topology_transition_rows",
        Some("topology_transitions"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_gateways",
        "WendaoGraph.topology_gateway_rows",
        Some("topology_gateways"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_community_summaries",
        "WendaoGraph.topology_community_summary_rows",
        Some("topology_community_summaries"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.topology_community_links",
        "WendaoGraph.topology_community_link_rows",
        Some("topology_community_links"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_community_frontier",
        "WendaoGraph.topology_community_frontier_rows",
        Some("topology_community_frontier"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.semantic_overlay",
        "WendaoGraph.semantic_overlay_edges",
        Some("semantic_overlay"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.diffusion_scores",
        "WendaoGraph.multi_plane_diffusion_scores",
        Some("diffusion_scores"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.link_frontier",
        "WendaoGraph.link_frontier_rows",
        Some("link_frontier"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
];

const WENDAO_GRAPH_RELATIONSHIP_SEARCH_ALGORITHMS: &[WendaoGraphAlgorithmRef] = &[
    relationship_search_algorithm(
        "relationship_search.hnsw_semantic_fanout",
        "WendaoGraph.hnsw_neighbor_rows",
        Some("semantic_neighbors"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.semantic_overlay_edges",
        "WendaoGraph.semantic_overlay_edges",
        Some("semantic_overlay"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    relationship_search_algorithm(
        "relationship_search.moc_community_grouping",
        "WendaoGraph.topology_community_rows",
        Some("topology_communities"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.community_bridge_links",
        "WendaoGraph.topology_community_link_rows",
        Some("topology_community_links"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.community_frontier_ranking",
        "WendaoGraph.topology_community_frontier_rows",
        Some("topology_community_frontier"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    relationship_search_algorithm(
        "relationship_search.ppr_like_relatedness",
        "WendaoGraph.multi_plane_diffusion_scores",
        Some("diffusion_scores"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.graph_search_ranking",
        "WendaoGraph.link_frontier_rows",
        Some("link_frontier"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    relationship_search_algorithm(
        "relationship_search.topology_candidate_ranking",
        "WendaoGraph.topology_candidate_rows",
        Some("topology_candidates"),
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.large_object_graph_traversal",
        "WendaoGraph.sparse_adjacency",
        None,
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.graph_snapshot_traversal",
        "WendaoGraph.build_graph_snapshot",
        None,
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
];

const WENDAO_GRAPH_PAGE_INDEX_ALGORITHMS: &[WendaoGraphAlgorithmRef] = &[
    page_index_algorithm(
        "page_index.reasoning_frontier",
        "WendaoGraph.page_index_reasoning_frontier_rows",
        Some("reasoning_frontier"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    page_index_algorithm(
        "page_index.disclosure_trace",
        "WendaoGraph.page_index_disclosure_trace_rows",
        Some("disclosure_trace"),
        WendaoGraphAlgorithmComplexity::Simple,
    ),
    page_index_algorithm(
        "page_index.planner_actions",
        "WendaoGraph.page_index_planner_action_rows",
        Some("page_index_planner_actions"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
];

const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ALGORITHMS: &[WendaoGraphAlgorithmRef] = &[
    search_strategy_flow_algorithm(
        "search_strategy_flow.candidate_rows",
        "WendaoGraph.strategy_flow_candidate_rows",
        Some("strategy_candidates"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    search_strategy_flow_algorithm(
        "search_strategy_flow.transition_rows",
        "WendaoGraph.strategy_flow_transition_rows",
        Some("strategy_transitions"),
        WendaoGraphAlgorithmComplexity::Simple,
    ),
    search_strategy_flow_algorithm(
        "search_strategy_flow.frontier_rows",
        "WendaoGraph.strategy_flow_frontier_rows",
        Some("strategy_frontier"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
    search_strategy_flow_algorithm(
        "search_strategy_flow.tables",
        "WendaoGraph.strategy_flow_tables",
        None,
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
];

const WENDAO_GRAPH_GNN_ALGORITHMS: &[WendaoGraphAlgorithmRef] = &[
    gnn_algorithm(
        "gnn.node_features",
        "WendaoGraph.gnn_node_features",
        None,
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    gnn_algorithm(
        "gnn.graph_tensor",
        "WendaoGraph.gnn_graph",
        None,
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    gnn_algorithm(
        "gnn.node_scores",
        "WendaoGraph.gnn_node_scores",
        None,
        WendaoGraphAlgorithmComplexity::Heavy,
    ),
    gnn_algorithm(
        "gnn.reasoning_frontier",
        "WendaoGraph.reasoning_frontier_rows",
        Some("reasoning_frontier"),
        WendaoGraphAlgorithmComplexity::Balanced,
    ),
];

/// Returns the `WendaoGraph.jl` `LinkGraph` algorithm catalog.
#[must_use]
pub const fn wendaograph_link_graph_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef] {
    WENDAO_GRAPH_LINK_GRAPH_ALGORITHMS
}

/// Returns the `WendaoGraph.jl` relationship-search algorithm catalog.
#[must_use]
pub const fn wendaograph_relationship_search_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef]
{
    WENDAO_GRAPH_RELATIONSHIP_SEARCH_ALGORITHMS
}

/// Returns the `WendaoGraph.jl` `PageIndex` algorithm catalog.
#[must_use]
pub const fn wendaograph_page_index_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef] {
    WENDAO_GRAPH_PAGE_INDEX_ALGORITHMS
}

/// Returns the `WendaoGraph.jl` `SearchStrategyFlow` algorithm catalog.
#[must_use]
pub const fn wendaograph_search_strategy_flow_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef]
{
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ALGORITHMS
}

/// Returns the `WendaoGraph.jl` GNN algorithm catalog.
#[must_use]
pub const fn wendaograph_gnn_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef] {
    WENDAO_GRAPH_GNN_ALGORITHMS
}

/// Returns all staged `WendaoGraph.jl` algorithm catalog entries.
#[must_use]
pub fn wendaograph_algorithm_refs() -> Vec<WendaoGraphAlgorithmRef> {
    let mut refs = Vec::with_capacity(
        WENDAO_GRAPH_LINK_GRAPH_ALGORITHMS.len()
            + WENDAO_GRAPH_RELATIONSHIP_SEARCH_ALGORITHMS.len()
            + WENDAO_GRAPH_PAGE_INDEX_ALGORITHMS.len()
            + WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ALGORITHMS.len()
            + WENDAO_GRAPH_GNN_ALGORITHMS.len(),
    );
    refs.extend_from_slice(WENDAO_GRAPH_LINK_GRAPH_ALGORITHMS);
    refs.extend_from_slice(WENDAO_GRAPH_RELATIONSHIP_SEARCH_ALGORITHMS);
    refs.extend_from_slice(WENDAO_GRAPH_PAGE_INDEX_ALGORITHMS);
    refs.extend_from_slice(WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ALGORITHMS);
    refs.extend_from_slice(WENDAO_GRAPH_GNN_ALGORITHMS);
    refs
}

/// Finds one staged `WendaoGraph.jl` algorithm catalog entry by id.
#[must_use]
pub fn wendaograph_algorithm_ref(
    algorithm_id: WendaoGraphAlgorithmId,
) -> Option<WendaoGraphAlgorithmRef> {
    wendaograph_algorithm_refs()
        .into_iter()
        .find(|reference| reference.algorithm_id == algorithm_id.0)
}

/// Returns the staged `WendaoGraph.jl` algorithm that owns one reasoning-tree
/// backend frontier evidence kind.
#[must_use]
pub fn wendaograph_frontier_algorithm_ref(evidence_kind: &str) -> Option<WendaoGraphAlgorithmRef> {
    let algorithm_id = match evidence_kind {
        "anchor_query" => "relationship_search.hnsw_semantic_fanout",
        "relation_path" => "relationship_search.ppr_like_relatedness",
        "page_index_seed" => "page_index.reasoning_frontier",
        "source_path" => "relationship_search.graph_search_ranking",
        _ => return None,
    };
    wendaograph_algorithm_ref(WendaoGraphAlgorithmId(algorithm_id))
}

/// Returns staged `WendaoGraph.jl` algorithm entries for one Julia profile id.
#[must_use]
pub fn wendaograph_algorithm_refs_for_profile(
    profile_id: WendaoGraphProfileId,
) -> Vec<WendaoGraphAlgorithmRef> {
    wendaograph_algorithm_refs()
        .into_iter()
        .filter(|reference| reference.profile_id == profile_id.0)
        .collect()
}

const fn link_graph_algorithm(
    algorithm_id: &'static str,
    julia_entrypoint: &'static str,
    output_table: Option<&'static str>,
    complexity: WendaoGraphAlgorithmComplexity,
) -> WendaoGraphAlgorithmRef {
    WendaoGraphAlgorithmRef::new(
        algorithm_id,
        "link_graph",
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
        julia_entrypoint,
        output_table,
        complexity,
    )
}

const fn page_index_algorithm(
    algorithm_id: &'static str,
    julia_entrypoint: &'static str,
    output_table: Option<&'static str>,
    complexity: WendaoGraphAlgorithmComplexity,
) -> WendaoGraphAlgorithmRef {
    WendaoGraphAlgorithmRef::new(
        algorithm_id,
        "page_index",
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
        julia_entrypoint,
        output_table,
        complexity,
    )
}

const fn relationship_search_algorithm(
    algorithm_id: &'static str,
    julia_entrypoint: &'static str,
    output_table: Option<&'static str>,
    complexity: WendaoGraphAlgorithmComplexity,
) -> WendaoGraphAlgorithmRef {
    WendaoGraphAlgorithmRef::new(
        algorithm_id,
        "relationship_search",
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
        julia_entrypoint,
        output_table,
        complexity,
    )
}

const fn search_strategy_flow_algorithm(
    algorithm_id: &'static str,
    julia_entrypoint: &'static str,
    output_table: Option<&'static str>,
    complexity: WendaoGraphAlgorithmComplexity,
) -> WendaoGraphAlgorithmRef {
    WendaoGraphAlgorithmRef::new(
        algorithm_id,
        "search_strategy_flow",
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
        julia_entrypoint,
        output_table,
        complexity,
    )
}

const fn gnn_algorithm(
    algorithm_id: &'static str,
    julia_entrypoint: &'static str,
    output_table: Option<&'static str>,
    complexity: WendaoGraphAlgorithmComplexity,
) -> WendaoGraphAlgorithmRef {
    WendaoGraphAlgorithmRef::new(
        algorithm_id,
        "gnn",
        WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
        julia_entrypoint,
        output_table,
        complexity,
    )
}
