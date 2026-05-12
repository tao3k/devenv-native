//! `link_graph::agentic::keys` owns Wendao link graph agentic keys behavior.

/// Valkey list key for passive suggested-link stream.
#[must_use]
pub fn suggested_link_stream_key(key_prefix: &str) -> String {
    format!("{key_prefix}:suggested_links:stream")
}

/// Valkey list key for suggested-link decision audit stream.
#[must_use]
pub fn suggested_link_decision_stream_key(key_prefix: &str) -> String {
    format!("{key_prefix}:suggested_links:decisions")
}
