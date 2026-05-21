//! `link_graph::context_snapshot::types` owns Wendao link graph context snapshot types behavior.

use crate::link_graph::models::{QuantumAnchorHit, QuantumContext, QuantumFusionOptions};
use crate::schemas::LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_V1;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Schema version identifier for persisted quantum-context snapshots.
pub const LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.link_graph.quantum_context_snapshot.v1";

static LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_FINGERPRINT: OnceLock<String> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Persisted snapshot of quantum-fusion contexts for rollback.
pub struct QuantumContextSnapshot {
    /// Schema version tag.
    pub schema_version: String,
    /// Schema fingerprint derived from canonical JSON schema.
    pub schema_fingerprint: String,
    /// Deterministic snapshot identifier.
    pub snapshot_id: String,
    /// Optional user query text associated with the snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_text: Option<String>,
    /// Snapshot creation time (unix seconds).
    pub saved_at_unix: f64,
    /// Anchor hits used to build the snapshot.
    pub anchors: Vec<QuantumAnchorHit>,
    /// Fusion options used to score contexts.
    pub options: QuantumFusionOptions,
    /// Fused contexts captured in this snapshot.
    pub contexts: Vec<QuantumContext>,
}

impl QuantumContextSnapshot {
    /// Build a snapshot payload from the provided inputs.
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub fn new(
        snapshot_id: impl Into<String>,
        query_text: Option<&str>,
        anchors: Vec<QuantumAnchorHit>,
        options: QuantumFusionOptions,
        contexts: Vec<QuantumContext>,
    ) -> Self {
        let saved_at_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |delta| delta.as_secs_f64());
        Self {
            schema_version: LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_VERSION.to_string(),
            schema_fingerprint: quantum_context_snapshot_schema_fingerprint().to_string(),
            snapshot_id: snapshot_id.into(),
            query_text: query_text
                .and_then(|value| {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    }
                })
                .map(str::to_string),
            saved_at_unix,
            anchors,
            options,
            contexts,
        }
    }
}

pub(crate) fn quantum_context_snapshot_schema_fingerprint() -> &'static str {
    LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_FINGERPRINT.get_or_init(|| {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_V1.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    })
}
