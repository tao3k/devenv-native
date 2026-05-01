use arrow::record_batch::RecordBatch;

/// One optional seed row sent to the `WendaoGraph` evidence request contract.
#[derive(Debug, Clone, PartialEq)]
pub struct WendaoGraphEvidenceSeed {
    /// Seed node id in the projected graph.
    pub node_id: String,
    /// Non-negative restart weight for the seed node.
    pub weight: f64,
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

/// Options controlling local `LinkGraphIndex` projection into `WendaoGraph` input tables.
#[derive(Debug, Clone, PartialEq)]
pub struct WendaoGraphEvidenceRequestOptions {
    /// Include `PageIndex` parent-child topology as structural graph edges.
    pub include_page_index: bool,
    /// Optional seed rows for diffusion/frontier evidence computation.
    pub seeds: Vec<WendaoGraphEvidenceSeed>,
}

impl Default for WendaoGraphEvidenceRequestOptions {
    fn default() -> Self {
        Self {
            include_page_index: true,
            seeds: Vec::new(),
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
    /// Optional `semantic_neighbors` table. Reserved for a later semantic overlay slice.
    pub semantic_neighbors: Option<RecordBatch>,
    /// Optional `semantic_overlay` table. Reserved for a later semantic overlay slice.
    pub semantic_overlay: Option<RecordBatch>,
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
}
