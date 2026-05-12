//! Saliency snapshot utilities for build-time integration.
//!
//! Provides batch fetching of saliency signals from Valkey during index build,
//! enabling knowledge distillation operators to access significance data.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::link_graph::index::build::saliency_snapshot::SaliencySnapshot;
//!
//! let snapshot = SaliencySnapshot::fetch(&node_ids, valkey_url, key_prefix)?;
//! let high_saliency = &snapshot.high_saliency_nodes;
//! ```

use crate::link_graph::saliency::{LinkGraphSaliencyState, valkey_saliency_get_many_with_valkey};
use std::collections::HashMap;

/// Threshold for considering a node as "high saliency".
pub const SALIENCY_THRESHOLD_HIGH: f64 = 0.70;

/// Minimum activation count for cluster membership eligibility.
pub const MIN_ACTIVATION_COUNT: u64 = 3;

/// Snapshot of saliency states captured at build time.
#[derive(Debug, Clone, Default)]
pub struct SaliencySnapshot {
    /// Map from `node_id` to saliency state.
    pub states: HashMap<String, LinkGraphSaliencyState>,
    /// Node IDs that exceed the high saliency threshold.
    pub high_saliency_nodes: Vec<String>,
}

impl SaliencySnapshot {
    /// Create an empty snapshot.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            states: HashMap::new(),
            high_saliency_nodes: Vec::new(),
        }
    }

    /// Fetch saliency states for the given node IDs.
    ///
    /// # Arguments
    /// * `node_ids` - List of document/node IDs to fetch saliency for
    /// * `valkey_url` - Valkey connection URL
    /// * `key_prefix` - Optional key prefix for namespacing
    ///
    /// # Errors
    /// Returns error if Valkey connection fails.
    pub fn fetch(
        node_ids: &[String],
        valkey_url: &str,
        key_prefix: Option<&str>,
    ) -> Result<Self, String> {
        let states = valkey_saliency_get_many_with_valkey(node_ids, valkey_url, key_prefix)?;

        let high_saliency_nodes: Vec<String> = states
            .iter()
            .filter(|(_, state)| {
                state.current_saliency >= SALIENCY_THRESHOLD_HIGH
                    && state.activation_count >= MIN_ACTIVATION_COUNT
            })
            .map(|(id, _)| id.clone())
            .collect();

        Ok(Self {
            states,
            high_saliency_nodes,
        })
    }

    /// Get saliency value for a specific node, defaulting to 0.0.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn saliency_of(&self, node_id: &str) -> f64 {
        self.states.get(node_id).map_or(0.0, |s| s.current_saliency)
    }

    /// Check if a node qualifies as high saliency.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn is_high_saliency(&self, node_id: &str) -> bool {
        self.high_saliency_nodes.contains(&node_id.to_string())
    }

    /// Get the count of nodes with any saliency data.
    #[must_use]
    pub fn known_count(&self) -> usize {
        self.states.len()
    }

    /// Get the count of high-saliency nodes.
    #[must_use]
    pub fn high_saliency_count(&self) -> usize {
        self.high_saliency_nodes.len()
    }

    /// Calculate average saliency across all known nodes.
    #[must_use]
    pub fn average_saliency(&self) -> f64 {
        if self.states.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.states.values().map(|s| s.current_saliency).sum();
        sum / usize_to_f64_saturating(self.states.len())
    }

    /// Get top N nodes by saliency.
    #[must_use]
    pub fn top_n(&self, n: usize) -> Vec<(&String, &LinkGraphSaliencyState)> {
        let mut sorted: Vec<_> = self.states.iter().collect();
        sorted.sort_by(|a, b| {
            b.1.current_saliency
                .partial_cmp(&a.1.current_saliency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(n).collect()
    }
}

fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}

#[cfg(test)]
#[path = "../../../../tests/unit/link_graph/index/build/saliency_snapshot.rs"]
mod tests;
