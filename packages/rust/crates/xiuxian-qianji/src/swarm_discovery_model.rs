use serde::{Deserialize, Serialize};

use super::discovery_util::normalize_optional_text;

/// Immutable identity published by one remote swarm worker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterNodeIdentity {
    /// Cluster identifier (for example region or deployment id).
    pub cluster_id: String,
    /// Agent identifier unique within one cluster.
    pub agent_id: String,
    /// Routing role class (for example `teacher`, `steward`).
    pub role_class: String,
    /// Optional region hint for cross-DC routing.
    pub region: Option<String>,
    /// Optional endpoint hint for remote invocation.
    pub endpoint: Option<String>,
    /// Optional capability labels published by this worker.
    pub capabilities: Vec<String>,
}

impl ClusterNodeIdentity {
    pub(in crate::swarm) fn sanitize(self) -> Self {
        Self {
            cluster_id: self.cluster_id.trim().to_string(),
            agent_id: self.agent_id.trim().to_string(),
            role_class: self.role_class.trim().to_ascii_lowercase(),
            region: normalize_optional_text(self.region),
            endpoint: normalize_optional_text(self.endpoint),
            capabilities: self
                .capabilities
                .into_iter()
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty())
                .collect(),
        }
    }
}

/// Resolved heartbeat record loaded from the global registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterNodeRecord {
    /// Registry key used by this record.
    pub registry_key: String,
    /// Published identity fields.
    pub identity: ClusterNodeIdentity,
    /// Last heartbeat timestamp in milliseconds.
    pub last_seen_ms: u64,
    /// Optional opaque metadata JSON published by the node.
    pub metadata: serde_json::Value,
}
