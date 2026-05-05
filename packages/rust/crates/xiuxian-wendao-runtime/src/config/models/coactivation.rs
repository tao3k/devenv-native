//! Coactivation runtime configuration records for related-node behavior.

use crate::config::constants::{
    DEFAULT_LINK_GRAPH_COACTIVATION_ALPHA_SCALE, DEFAULT_LINK_GRAPH_COACTIVATION_ENABLED,
    DEFAULT_LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE, DEFAULT_LINK_GRAPH_COACTIVATION_MAX_HOPS,
    DEFAULT_LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION,
    DEFAULT_LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH,
};

/// Resolved runtime controls for link-graph coactivation propagation.
#[derive(Debug, Clone, Copy)]
pub struct LinkGraphCoactivationRuntimeConfig {
    /// Whether coactivation propagation is enabled.
    pub enabled: bool,
    /// Alpha scale applied to coactivation scoring.
    pub alpha_scale: f64,
    /// Neighbor fanout applied per graph direction.
    pub max_neighbors_per_direction: usize,
    /// Maximum traversal depth.
    pub max_hops: usize,
    /// Maximum number of propagation operations allowed.
    pub max_total_propagations: usize,
    /// Hop-to-hop decay scale.
    pub hop_decay_scale: f64,
    /// Queue depth used for touch staging.
    pub touch_queue_depth: usize,
}

impl Default for LinkGraphCoactivationRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_LINK_GRAPH_COACTIVATION_ENABLED,
            alpha_scale: DEFAULT_LINK_GRAPH_COACTIVATION_ALPHA_SCALE,
            max_neighbors_per_direction:
                DEFAULT_LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION,
            max_hops: DEFAULT_LINK_GRAPH_COACTIVATION_MAX_HOPS,
            max_total_propagations: DEFAULT_LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION
                .saturating_mul(2),
            hop_decay_scale: DEFAULT_LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE,
            touch_queue_depth: DEFAULT_LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH,
        }
    }
}
