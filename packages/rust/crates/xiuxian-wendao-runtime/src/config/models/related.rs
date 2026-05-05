//! Related-node runtime configuration records for Wendao expansion behavior.

use crate::config::constants::{
    DEFAULT_LINK_GRAPH_RELATED_MAX_CANDIDATES, DEFAULT_LINK_GRAPH_RELATED_MAX_PARTITIONS,
    DEFAULT_LINK_GRAPH_RELATED_TIME_BUDGET_MS,
};

/// Resolved runtime limits for related-query execution.
#[derive(Debug, Clone, Copy)]
pub struct LinkGraphRelatedRuntimeConfig {
    /// Maximum candidate rows gathered before reranking.
    pub max_candidates: usize,
    /// Maximum partitions scanned during related-query fanout.
    pub max_partitions: usize,
    /// Time budget for related-query execution, in milliseconds.
    pub time_budget_ms: f64,
}

impl Default for LinkGraphRelatedRuntimeConfig {
    fn default() -> Self {
        Self {
            max_candidates: DEFAULT_LINK_GRAPH_RELATED_MAX_CANDIDATES,
            max_partitions: DEFAULT_LINK_GRAPH_RELATED_MAX_PARTITIONS,
            time_budget_ms: DEFAULT_LINK_GRAPH_RELATED_TIME_BUDGET_MS,
        }
    }
}
