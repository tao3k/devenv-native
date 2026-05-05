//! Read-only projection rows for host compute and memory inspection lanes.

use crate::episode::Episode;
use serde::{Deserialize, Serialize};

/// Read-only filter applied when materializing a host memory projection.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryProjectionFilter {
    /// Restrict the projection to one logical scope.
    pub scope: Option<String>,
    /// Limit the number of rows returned after filtering.
    pub limit: Option<usize>,
}

impl MemoryProjectionFilter {
    /// Return the normalized scope key used by `EpisodeStore`.
    #[must_use]
    pub fn normalized_scope(&self) -> Option<String> {
        self.scope.as_deref().map(Episode::normalize_scope)
    }
}

/// Canonical read-only episode features exported to Julia compute lanes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryProjectionRow {
    /// Stable episode identifier used as the host-side join key.
    pub episode_id: String,
    /// Logical scope associated with the episode.
    pub scope: String,
    /// Semantic embedding of the stored intent.
    pub intent_embedding: Vec<f32>,
    /// Current utility estimate sourced from the host Q-table.
    pub q_value: f32,
    /// Number of observed successful recalls.
    pub success_count: u32,
    /// Number of observed failed recalls.
    pub failure_count: u32,
    /// Number of total retrievals or accesses.
    pub retrieval_count: u32,
    /// Episode creation timestamp in Unix milliseconds.
    pub created_at_ms: i64,
    /// Episode last-update timestamp in Unix milliseconds.
    pub updated_at_ms: i64,
}

impl MemoryProjectionRow {
    #[must_use]
    pub(crate) fn from_episode(episode: &Episode, q_value: f32) -> Self {
        Self {
            episode_id: episode.id.clone(),
            scope: episode.scope.clone(),
            intent_embedding: episode.intent_embedding.clone(),
            q_value,
            success_count: episode.success_count,
            failure_count: episode.failure_count,
            retrieval_count: episode.retrieval_count,
            created_at_ms: episode.created_at,
            updated_at_ms: episode.updated_at,
        }
    }
}
