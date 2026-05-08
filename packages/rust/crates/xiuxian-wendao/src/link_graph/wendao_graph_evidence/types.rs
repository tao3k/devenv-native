use arrow::record_batch::RecordBatch;

/// One optional seed row sent to the `WendaoGraph` evidence request contract.
#[derive(Debug, Clone, PartialEq)]
pub struct WendaoGraphEvidenceSeed {
    /// Seed node id in the projected graph.
    pub node_id: String,
    /// Non-negative restart weight for the seed node.
    pub weight: f64,
}

/// One optional semantic-neighbor row sent to the `WendaoGraph` evidence request contract.
#[derive(Debug, Clone, PartialEq)]
pub struct WendaoGraphSemanticNeighbor {
    /// Query node id in the projected graph.
    pub query_id: String,
    /// Neighbor node id in the projected graph.
    pub neighbor_id: String,
    /// One-based query vertex index expected by `WendaoGraph.jl`.
    pub query_index: i64,
    /// One-based neighbor vertex index expected by `WendaoGraph.jl`.
    pub neighbor_index: i64,
    /// Positive rank within the query's semantic-neighbor list.
    pub rank: i64,
    /// Non-negative semantic distance.
    pub distance: f64,
}

/// One optional semantic-overlay row sent to the `WendaoGraph` evidence request contract.
#[derive(Debug, Clone, PartialEq)]
pub struct WendaoGraphSemanticOverlayEdge {
    /// Source node id in the projected graph.
    pub source_id: String,
    /// Target node id in the projected graph.
    pub target_id: String,
    /// One-based source vertex index expected by `WendaoGraph.jl`.
    pub source_index: i64,
    /// One-based target vertex index expected by `WendaoGraph.jl`.
    pub target_index: i64,
    /// Positive rank within the semantic overlay for the source node.
    pub rank: i64,
    /// Non-negative semantic distance.
    pub distance: f64,
    /// Non-negative overlay edge weight.
    pub weight: f64,
    /// Overlay edge classification consumed by `WendaoGraph.jl`.
    pub edge_kind: String,
}

/// One optional seed row sent to the `WendaoGraph` `PageIndex` reasoning contract.
#[derive(Debug, Clone, PartialEq)]
pub struct WendaoGraphPageIndexReasoningSeed {
    /// `PageIndex` node id.
    pub node_id: String,
    /// Non-negative weight for the seed node.
    pub weight: f64,
    /// Seed classification consumed by `WendaoGraph.jl`.
    pub seed_kind: String,
}

impl WendaoGraphPageIndexReasoningSeed {
    /// Create a `PageIndex` reasoning seed row.
    #[must_use]
    pub fn new(node_id: impl Into<String>, weight: f64, seed_kind: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            weight,
            seed_kind: seed_kind.into(),
        }
    }
}

impl WendaoGraphEvidenceSeed {
    /// Create a seed row.
    #[must_use]
    pub fn new(node_id: impl Into<String>, weight: f64) -> Self {
        Self {
            node_id: node_id.into(),
            weight,
        }
    }
}

impl WendaoGraphSemanticNeighbor {
    /// Create a semantic-neighbor row.
    #[must_use]
    pub fn new(
        query_id: impl Into<String>,
        neighbor_id: impl Into<String>,
        query_index: i64,
        neighbor_index: i64,
        rank: i64,
        distance: f64,
    ) -> Self {
        Self {
            query_id: query_id.into(),
            neighbor_id: neighbor_id.into(),
            query_index,
            neighbor_index,
            rank,
            distance,
        }
    }
}

/// Options controlling local `LinkGraphIndex` projection into `WendaoGraph` input tables.
#[derive(Debug, Clone, PartialEq)]
pub struct WendaoGraphEvidenceRequestOptions {
    /// Include `PageIndex` parent-child topology as structural graph edges.
    pub include_page_index: bool,
    /// Optional seed rows for diffusion/frontier evidence computation.
    pub seeds: Vec<WendaoGraphEvidenceSeed>,
    /// Optional semantic-neighbor rows for semantic overlay and diffusion.
    pub semantic_neighbors: Vec<WendaoGraphSemanticNeighbor>,
    /// Optional precomputed semantic-overlay rows for diffusion/frontier evidence.
    pub semantic_overlay: Vec<WendaoGraphSemanticOverlayEdge>,
}

/// Options controlling local `LinkGraphIndex` projection into `PageIndex` reasoning input tables.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WendaoGraphPageIndexReasoningRequestOptions {
    /// Optional seed rows for `PageIndex` reasoning frontier construction.
    pub seeds: Vec<WendaoGraphPageIndexReasoningSeed>,
}

impl WendaoGraphPageIndexReasoningRequestOptions {
    /// Append one seed row.
    #[must_use]
    pub fn with_seed(
        mut self,
        node_id: impl Into<String>,
        weight: f64,
        seed_kind: impl Into<String>,
    ) -> Self {
        self.seeds.push(WendaoGraphPageIndexReasoningSeed::new(
            node_id, weight, seed_kind,
        ));
        self
    }
}

impl Default for WendaoGraphEvidenceRequestOptions {
    fn default() -> Self {
        Self {
            include_page_index: true,
            seeds: Vec::new(),
            semantic_neighbors: Vec::new(),
            semantic_overlay: Vec::new(),
        }
    }
}

impl WendaoGraphEvidenceRequestOptions {
    /// Return options with `PageIndex` projection disabled.
    #[must_use]
    pub fn without_page_index(mut self) -> Self {
        self.include_page_index = false;
        self
    }

    /// Append one seed row.
    #[must_use]
    pub fn with_seed(mut self, node_id: impl Into<String>, weight: f64) -> Self {
        self.seeds
            .push(WendaoGraphEvidenceSeed::new(node_id, weight));
        self
    }

    /// Append one semantic-neighbor row.
    #[must_use]
    pub fn with_semantic_neighbor(
        mut self,
        query_id: impl Into<String>,
        neighbor_id: impl Into<String>,
        query_index: i64,
        neighbor_index: i64,
        rank: i64,
        distance: f64,
    ) -> Self {
        self.semantic_neighbors
            .push(WendaoGraphSemanticNeighbor::new(
                query_id,
                neighbor_id,
                query_index,
                neighbor_index,
                rank,
                distance,
            ));
        self
    }

    /// Append one precomputed semantic-overlay row.
    #[must_use]
    pub fn with_semantic_overlay_edge(mut self, edge: WendaoGraphSemanticOverlayEdge) -> Self {
        self.semantic_overlay.push(edge);
        self
    }
}

/// Validated host-to-WendaoGraph request tables in canonical table order.
#[derive(Debug, Clone)]
pub struct WendaoGraphEvidenceRequestBundle {
    /// Required `nodes` table.
    pub nodes: RecordBatch,
    /// Required `edges` table.
    pub edges: RecordBatch,
    /// Optional `seeds` table.
    pub seeds: Option<RecordBatch>,
    /// Optional `semantic_neighbors` table used by Julia to derive semantic overlay evidence.
    pub semantic_neighbors: Option<RecordBatch>,
    /// Optional `semantic_overlay` table with host-precomputed semantic overlay evidence.
    pub semantic_overlay: Option<RecordBatch>,
}

/// Validated host-to-WendaoGraph `PageIndex` reasoning request tables.
#[derive(Debug, Clone)]
pub struct WendaoGraphPageIndexReasoningRequestBundle {
    /// Required `page_index_nodes` table.
    pub nodes: RecordBatch,
    /// Required `page_index_edges` table.
    pub edges: RecordBatch,
    /// Optional-by-semantics `page_index_seeds` table, always materialized for schema stability.
    pub seeds: RecordBatch,
}

impl WendaoGraphPageIndexReasoningRequestBundle {
    /// Return available request tables by canonical name and order.
    #[must_use]
    pub fn record_batches(&self) -> Vec<(&'static str, &RecordBatch)> {
        vec![
            ("page_index_nodes", &self.nodes),
            ("page_index_edges", &self.edges),
            ("page_index_seeds", &self.seeds),
        ]
    }

    /// Return an available table by canonical table name.
    #[must_use]
    pub fn table(&self, table_name: &str) -> Option<&RecordBatch> {
        match table_name {
            "page_index_nodes" => Some(&self.nodes),
            "page_index_edges" => Some(&self.edges),
            "page_index_seeds" => Some(&self.seeds),
            _ => None,
        }
    }

    /// Consume the bundle into canonical table-name and batch pairs.
    #[must_use]
    pub fn into_named_record_batches(self) -> Vec<(&'static str, RecordBatch)> {
        vec![
            ("page_index_nodes", self.nodes),
            ("page_index_edges", self.edges),
            ("page_index_seeds", self.seeds),
        ]
    }

    /// Consume the bundle into canonical request batches without table names.
    #[must_use]
    pub fn into_record_batches(self) -> Vec<RecordBatch> {
        self.into_named_record_batches()
            .into_iter()
            .map(|(_, batch)| batch)
            .collect()
    }
}

impl WendaoGraphEvidenceRequestBundle {
    /// Return available request tables by canonical name and order.
    #[must_use]
    pub fn record_batches(&self) -> Vec<(&'static str, &RecordBatch)> {
        let mut batches = vec![("nodes", &self.nodes), ("edges", &self.edges)];
        if let Some(seeds) = &self.seeds {
            batches.push(("seeds", seeds));
        }
        if let Some(semantic_neighbors) = &self.semantic_neighbors {
            batches.push(("semantic_neighbors", semantic_neighbors));
        }
        if let Some(semantic_overlay) = &self.semantic_overlay {
            batches.push(("semantic_overlay", semantic_overlay));
        }
        batches
    }

    /// Return an available table by canonical table name.
    #[must_use]
    pub fn table(&self, table_name: &str) -> Option<&RecordBatch> {
        match table_name {
            "nodes" => Some(&self.nodes),
            "edges" => Some(&self.edges),
            "seeds" => self.seeds.as_ref(),
            "semantic_neighbors" => self.semantic_neighbors.as_ref(),
            "semantic_overlay" => self.semantic_overlay.as_ref(),
            _ => None,
        }
    }

    /// Consume the bundle into canonical table-name and batch pairs.
    #[must_use]
    pub fn into_named_record_batches(self) -> Vec<(&'static str, RecordBatch)> {
        let mut batches = vec![("nodes", self.nodes), ("edges", self.edges)];
        if let Some(seeds) = self.seeds {
            batches.push(("seeds", seeds));
        }
        if let Some(semantic_neighbors) = self.semantic_neighbors {
            batches.push(("semantic_neighbors", semantic_neighbors));
        }
        if let Some(semantic_overlay) = self.semantic_overlay {
            batches.push(("semantic_overlay", semantic_overlay));
        }
        batches
    }

    /// Consume the bundle into canonical request batches without table names.
    #[must_use]
    pub fn into_record_batches(self) -> Vec<RecordBatch> {
        self.into_named_record_batches()
            .into_iter()
            .map(|(_, batch)| batch)
            .collect()
    }
}

/// Errors raised while projecting `LinkGraphIndex` into `WendaoGraph` request tables.
#[derive(Debug, thiserror::Error)]
pub enum LinkGraphWendaoGraphEvidenceError {
    /// A seed row uses a blank node id.
    #[error("WendaoGraph evidence seed node id must not be blank")]
    BlankSeedNode,
    /// A seed references a node missing from the projected graph.
    #[error(
        "WendaoGraph evidence seed `{node_id}` references a node that is not present in the projected graph"
    )]
    UnknownSeedNode {
        /// Missing seed node id.
        node_id: String,
    },
    /// A seed has a negative or non-finite weight.
    #[error("WendaoGraph evidence seed `{node_id}` weight must be finite and non-negative")]
    InvalidSeedWeight {
        /// Seed node id.
        node_id: String,
    },
    /// A semantic-neighbor row references a node missing from the projected graph.
    #[error(
        "WendaoGraph semantic neighbor `{node_id}` references a node that is not present in the projected graph"
    )]
    UnknownSemanticNeighborNode {
        /// Missing node id.
        node_id: String,
    },
    /// A semantic-neighbor row has an invalid index, rank, or distance.
    #[error("WendaoGraph semantic neighbor `{query_id}` -> `{neighbor_id}` has invalid {field}")]
    InvalidSemanticNeighbor {
        /// Query node id.
        query_id: String,
        /// Neighbor node id.
        neighbor_id: String,
        /// Invalid field name.
        field: &'static str,
    },
    /// A request tried to provide both optional semantic input variants.
    #[error(
        "WendaoGraph evidence request must provide semantic_neighbors or semantic_overlay, not both"
    )]
    ConflictingSemanticEvidence,
    /// A semantic-overlay row references a node missing from the projected graph.
    #[error(
        "WendaoGraph semantic overlay `{node_id}` references a node that is not present in the projected graph"
    )]
    UnknownSemanticOverlayNode {
        /// Missing node id.
        node_id: String,
    },
    /// A semantic-overlay row has an invalid index, rank, distance, weight, or edge kind.
    #[error("WendaoGraph semantic overlay `{source_id}` -> `{target_id}` has invalid {field}")]
    InvalidSemanticOverlay {
        /// Source node id.
        source_id: String,
        /// Target node id.
        target_id: String,
        /// Invalid field name.
        field: &'static str,
    },
    /// A `PageIndex` seed has a blank seed kind.
    #[error("WendaoGraph PageIndex reasoning seed `{node_id}` seed_kind must not be blank")]
    BlankSeedKind {
        /// Seed node id.
        node_id: String,
    },
    /// A usize value did not fit into the Arrow Int64 contract.
    #[error(
        "WendaoGraph evidence `{table_name}` column `{column}` value `{value}` does not fit Int64"
    )]
    IntegerOverflow {
        /// Canonical request table name.
        table_name: &'static str,
        /// Column name.
        column: &'static str,
        /// Source value.
        value: usize,
    },
    /// Arrow batch construction failed.
    #[error("failed to build WendaoGraph evidence `{table_name}` batch: {message}")]
    Batch {
        /// Canonical request table name.
        table_name: &'static str,
        /// Underlying Arrow error message.
        message: String,
    },
    /// Contract schema validation failed.
    #[error("WendaoGraph evidence `{table_name}` schema validation failed: {message}")]
    Schema {
        /// Canonical request table name.
        table_name: &'static str,
        /// Validation error message.
        message: String,
    },
    /// A semantic relation references a node outside the projected semantic scope.
    #[error("WendaoGraph semantic reasoning relation references missing node `{node_id}`")]
    SemanticRelationMissingNode {
        /// Missing semantic node id.
        node_id: String,
    },
    /// Semantic containment edges form a cycle and cannot produce a deterministic `PageIndex` depth.
    #[error("WendaoGraph semantic reasoning containment cycle reaches `{node_id}`")]
    SemanticContainmentCycle {
        /// Node id where cycle detection was requested.
        node_id: String,
    },
}
