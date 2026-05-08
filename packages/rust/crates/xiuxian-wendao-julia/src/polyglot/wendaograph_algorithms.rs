//! Static `WendaoGraph.jl` algorithm catalog for owner-side scheduling evidence.

use xiuxian_polyglot_orchestrator::{
    JuliaComputeTaskShape, JuliaTaskComplexityClass, LaneCapability,
};

use super::{
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
};

/// Inert catalog entry for one `WendaoGraph.jl` algorithm visible to Rust.
///
/// The catalog is descriptive owner evidence. It does not call Julia, validate
/// a route, or decide admission by itself.
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
    /// Capability class used by the Rust scheduler.
    pub capability: LaneCapability,
    /// Owner-supplied scheduler complexity hint.
    pub complexity: JuliaTaskComplexityClass,
}

impl WendaoGraphAlgorithmRef {
    /// Creates an inert `WendaoGraph.jl` algorithm catalog entry.
    #[must_use]
    pub const fn new(
        algorithm_id: &'static str,
        family: &'static str,
        profile_id: &'static str,
        julia_entrypoint: &'static str,
        output_table: Option<&'static str>,
        capability: LaneCapability,
        complexity: JuliaTaskComplexityClass,
    ) -> Self {
        Self {
            algorithm_id,
            family,
            profile_id,
            julia_entrypoint,
            output_table,
            capability,
            complexity,
        }
    }

    /// Returns whether this algorithm is marked as structurally heavy.
    #[must_use]
    pub const fn is_heavy(self) -> bool {
        matches!(self.complexity, JuliaTaskComplexityClass::Heavy)
    }

    /// Returns a scheduler task shape for this algorithm and workload.
    #[must_use]
    pub fn task_shape(self, workload: WendaoGraphAlgorithmWorkload) -> JuliaComputeTaskShape {
        JuliaComputeTaskShape::new()
            .with_rows(workload.rows.max(1))
            .with_graph_size(workload.nodes, workload.edges)
            .with_feature_columns(workload.feature_columns)
            .with_byte_size(workload.byte_size)
            .with_batchability_key(format!(
                "wendaograph:{}:{}",
                self.profile_id, self.algorithm_id
            ))
            .with_complexity(self.complexity)
    }
}

/// Owner-supplied workload facts for one `WendaoGraph.jl` algorithm request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WendaoGraphAlgorithmWorkload {
    /// Logical rows or candidate items in the request.
    pub rows: u32,
    /// Graph node count relevant to the request.
    pub nodes: u32,
    /// Graph edge count relevant to the request.
    pub edges: u32,
    /// Feature or signal columns relevant to the request.
    pub feature_columns: u32,
    /// Estimated serialized input bytes.
    pub byte_size: u64,
}

impl WendaoGraphAlgorithmWorkload {
    /// Creates an empty algorithm workload.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rows: 1,
            nodes: 0,
            edges: 0,
            feature_columns: 0,
            byte_size: 0,
        }
    }

    /// Returns this workload with logical row count.
    #[must_use]
    pub const fn with_rows(mut self, rows: u32) -> Self {
        self.rows = rows;
        self
    }

    /// Returns this workload with graph node and edge counts.
    #[must_use]
    pub const fn with_graph_size(mut self, nodes: u32, edges: u32) -> Self {
        self.nodes = nodes;
        self.edges = edges;
        self
    }

    /// Returns this workload with feature column count.
    #[must_use]
    pub const fn with_feature_columns(mut self, feature_columns: u32) -> Self {
        self.feature_columns = feature_columns;
        self
    }

    /// Returns this workload with estimated serialized byte size.
    #[must_use]
    pub const fn with_byte_size(mut self, byte_size: u64) -> Self {
        self.byte_size = byte_size;
        self
    }
}

impl Default for WendaoGraphAlgorithmWorkload {
    fn default() -> Self {
        Self::new()
    }
}

const WENDAO_GRAPH_LINK_GRAPH_ALGORITHMS: &[WendaoGraphAlgorithmRef] = &[
    link_graph_algorithm(
        "link_graph.graph_metrics",
        "WendaoGraph.graph_metric_rows",
        Some("graph_metrics"),
        JuliaTaskComplexityClass::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.components",
        "WendaoGraph.component_rows",
        Some("components"),
        JuliaTaskComplexityClass::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.topology_profile",
        "WendaoGraph.topology_profile_rows",
        Some("topology_profile"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_candidates",
        "WendaoGraph.topology_candidate_rows",
        Some("topology_candidates"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_bottlenecks",
        "WendaoGraph.topology_bottleneck_rows",
        Some("topology_bottlenecks"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_communities",
        "WendaoGraph.topology_community_rows",
        Some("topology_communities"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_cover",
        "WendaoGraph.topology_cover_rows",
        Some("topology_cover"),
        JuliaTaskComplexityClass::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.topology_core",
        "WendaoGraph.topology_core_rows",
        Some("topology_core"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_boundary",
        "WendaoGraph.topology_boundary_rows",
        Some("topology_boundary"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_transitions",
        "WendaoGraph.topology_transition_rows",
        Some("topology_transitions"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_gateways",
        "WendaoGraph.topology_gateway_rows",
        Some("topology_gateways"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_community_summaries",
        "WendaoGraph.topology_community_summary_rows",
        Some("topology_community_summaries"),
        JuliaTaskComplexityClass::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.topology_community_links",
        "WendaoGraph.topology_community_link_rows",
        Some("topology_community_links"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.topology_community_frontier",
        "WendaoGraph.topology_community_frontier_rows",
        Some("topology_community_frontier"),
        JuliaTaskComplexityClass::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.semantic_overlay",
        "WendaoGraph.semantic_overlay_edges",
        Some("semantic_overlay"),
        JuliaTaskComplexityClass::Balanced,
    ),
    link_graph_algorithm(
        "link_graph.diffusion_scores",
        "WendaoGraph.multi_plane_diffusion_scores",
        Some("diffusion_scores"),
        JuliaTaskComplexityClass::Heavy,
    ),
    link_graph_algorithm(
        "link_graph.link_frontier",
        "WendaoGraph.link_frontier_rows",
        Some("link_frontier"),
        JuliaTaskComplexityClass::Balanced,
    ),
];

const WENDAO_GRAPH_RELATIONSHIP_SEARCH_ALGORITHMS: &[WendaoGraphAlgorithmRef] = &[
    relationship_search_algorithm(
        "relationship_search.hnsw_semantic_fanout",
        "WendaoGraph.hnsw_neighbor_rows",
        Some("semantic_neighbors"),
        JuliaTaskComplexityClass::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.semantic_overlay_edges",
        "WendaoGraph.semantic_overlay_edges",
        Some("semantic_overlay"),
        JuliaTaskComplexityClass::Balanced,
    ),
    relationship_search_algorithm(
        "relationship_search.moc_community_grouping",
        "WendaoGraph.topology_community_rows",
        Some("topology_communities"),
        JuliaTaskComplexityClass::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.community_bridge_links",
        "WendaoGraph.topology_community_link_rows",
        Some("topology_community_links"),
        JuliaTaskComplexityClass::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.community_frontier_ranking",
        "WendaoGraph.topology_community_frontier_rows",
        Some("topology_community_frontier"),
        JuliaTaskComplexityClass::Balanced,
    ),
    relationship_search_algorithm(
        "relationship_search.ppr_like_relatedness",
        "WendaoGraph.multi_plane_diffusion_scores",
        Some("diffusion_scores"),
        JuliaTaskComplexityClass::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.graph_search_ranking",
        "WendaoGraph.link_frontier_rows",
        Some("link_frontier"),
        JuliaTaskComplexityClass::Balanced,
    ),
    relationship_search_algorithm(
        "relationship_search.topology_candidate_ranking",
        "WendaoGraph.topology_candidate_rows",
        Some("topology_candidates"),
        JuliaTaskComplexityClass::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.large_object_graph_traversal",
        "WendaoGraph.sparse_adjacency",
        None,
        JuliaTaskComplexityClass::Heavy,
    ),
    relationship_search_algorithm(
        "relationship_search.graph_snapshot_traversal",
        "WendaoGraph.build_graph_snapshot",
        None,
        JuliaTaskComplexityClass::Heavy,
    ),
];

const WENDAO_GRAPH_PAGE_INDEX_ALGORITHMS: &[WendaoGraphAlgorithmRef] = &[
    page_index_algorithm(
        "page_index.reasoning_frontier",
        "WendaoGraph.page_index_reasoning_frontier_rows",
        Some("reasoning_frontier"),
        JuliaTaskComplexityClass::Balanced,
    ),
    page_index_algorithm(
        "page_index.disclosure_trace",
        "WendaoGraph.page_index_disclosure_trace_rows",
        Some("disclosure_trace"),
        JuliaTaskComplexityClass::Simple,
    ),
    page_index_algorithm(
        "page_index.planner_actions",
        "WendaoGraph.page_index_planner_action_rows",
        Some("page_index_planner_actions"),
        JuliaTaskComplexityClass::Balanced,
    ),
];

const WENDAO_GRAPH_GNN_ALGORITHMS: &[WendaoGraphAlgorithmRef] = &[
    gnn_algorithm(
        "gnn.node_features",
        "WendaoGraph.gnn_node_features",
        None,
        JuliaTaskComplexityClass::Heavy,
    ),
    gnn_algorithm(
        "gnn.graph_tensor",
        "WendaoGraph.gnn_graph",
        None,
        JuliaTaskComplexityClass::Heavy,
    ),
    gnn_algorithm(
        "gnn.node_scores",
        "WendaoGraph.gnn_node_scores",
        None,
        JuliaTaskComplexityClass::Heavy,
    ),
    gnn_algorithm(
        "gnn.reasoning_frontier",
        "WendaoGraph.reasoning_frontier_rows",
        Some("reasoning_frontier"),
        JuliaTaskComplexityClass::Balanced,
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
            + WENDAO_GRAPH_GNN_ALGORITHMS.len(),
    );
    refs.extend_from_slice(WENDAO_GRAPH_LINK_GRAPH_ALGORITHMS);
    refs.extend_from_slice(WENDAO_GRAPH_RELATIONSHIP_SEARCH_ALGORITHMS);
    refs.extend_from_slice(WENDAO_GRAPH_PAGE_INDEX_ALGORITHMS);
    refs.extend_from_slice(WENDAO_GRAPH_GNN_ALGORITHMS);
    refs
}

/// Finds one staged `WendaoGraph.jl` algorithm catalog entry by id.
#[must_use]
pub fn wendaograph_algorithm_ref(algorithm_id: &str) -> Option<WendaoGraphAlgorithmRef> {
    wendaograph_algorithm_refs()
        .into_iter()
        .find(|reference| reference.algorithm_id == algorithm_id)
}

/// Returns the scheduler task shape for one staged `WendaoGraph.jl`
/// algorithm id and workload.
#[must_use]
pub fn wendaograph_algorithm_task_shape(
    algorithm_id: &str,
    workload: WendaoGraphAlgorithmWorkload,
) -> Option<JuliaComputeTaskShape> {
    wendaograph_algorithm_ref(algorithm_id).map(|reference| reference.task_shape(workload))
}

/// Returns staged `WendaoGraph.jl` algorithm entries for one Julia profile id.
#[must_use]
pub fn wendaograph_algorithm_refs_for_profile(profile_id: &str) -> Vec<WendaoGraphAlgorithmRef> {
    wendaograph_algorithm_refs()
        .into_iter()
        .filter(|reference| reference.profile_id == profile_id)
        .collect()
}

const fn link_graph_algorithm(
    algorithm_id: &'static str,
    julia_entrypoint: &'static str,
    output_table: Option<&'static str>,
    complexity: JuliaTaskComplexityClass,
) -> WendaoGraphAlgorithmRef {
    WendaoGraphAlgorithmRef::new(
        algorithm_id,
        "link_graph",
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
        julia_entrypoint,
        output_table,
        LaneCapability::GraphEvidenceCompute,
        complexity,
    )
}

const fn page_index_algorithm(
    algorithm_id: &'static str,
    julia_entrypoint: &'static str,
    output_table: Option<&'static str>,
    complexity: JuliaTaskComplexityClass,
) -> WendaoGraphAlgorithmRef {
    WendaoGraphAlgorithmRef::new(
        algorithm_id,
        "page_index",
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
        julia_entrypoint,
        output_table,
        LaneCapability::GraphEvidenceCompute,
        complexity,
    )
}

const fn relationship_search_algorithm(
    algorithm_id: &'static str,
    julia_entrypoint: &'static str,
    output_table: Option<&'static str>,
    complexity: JuliaTaskComplexityClass,
) -> WendaoGraphAlgorithmRef {
    WendaoGraphAlgorithmRef::new(
        algorithm_id,
        "relationship_search",
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
        julia_entrypoint,
        output_table,
        LaneCapability::GraphEvidenceCompute,
        complexity,
    )
}

const fn gnn_algorithm(
    algorithm_id: &'static str,
    julia_entrypoint: &'static str,
    output_table: Option<&'static str>,
    complexity: JuliaTaskComplexityClass,
) -> WendaoGraphAlgorithmRef {
    WendaoGraphAlgorithmRef::new(
        algorithm_id,
        "gnn",
        WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
        julia_entrypoint,
        output_table,
        LaneCapability::GraphEvidenceCompute,
        complexity,
    )
}
