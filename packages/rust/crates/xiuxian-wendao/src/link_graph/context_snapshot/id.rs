//! `link_graph::context_snapshot::id` owns Wendao link graph context snapshot id behavior.

use crate::link_graph::context_snapshot::types::LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_VERSION;
use crate::link_graph::models::{QuantumAnchorHit, QuantumFusionOptions};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Serialize)]
struct QuantumContextSnapshotKey<'a> {
    query_text: &'a str,
    anchors: &'a [QuantumAnchorHit],
    options: &'a QuantumFusionOptions,
}

/// Compute a deterministic snapshot id for a query + anchors payload.
///
/// # Errors
///
/// Returns an error when the query text is empty or JSON serialization fails.
pub fn quantum_context_snapshot_id(
    query_text: &str,
    anchors: &[QuantumAnchorHit],
    options: &QuantumFusionOptions,
) -> Result<String, String> {
    let trimmed = query_text.trim();
    if trimmed.is_empty() {
        return Err("query_text must be non-empty".to_string());
    }
    let payload = QuantumContextSnapshotKey {
        query_text: trimmed,
        anchors,
        options,
    };
    let encoded = serde_json::to_string(&payload)
        .map_err(|err| format!("failed to encode quantum-context snapshot key: {err}"))?;
    let mut hasher = DefaultHasher::new();
    encoded.hash(&mut hasher);
    LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_VERSION.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
}
