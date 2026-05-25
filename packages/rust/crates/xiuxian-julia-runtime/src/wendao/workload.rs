//! Stable Wendao-facing Julia workload descriptors.

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
