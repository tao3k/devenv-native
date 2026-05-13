//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
//! `link_graph::agentic::store::common` owns Wendao agentic store common behavior.

use crate::link_graph::agentic::types::LinkGraphSuggestedLinkState;
use crate::valkey_common::open_client;
use std::time::{SystemTime, UNIX_EPOCH};
use xxhash_rust::xxh3::xxh3_64;

/// Build a Redis client for suggested-link store operations.
///
/// # Errors
/// Returns an error when the provided Valkey URL is invalid.
pub fn redis_client(valkey_url: &str) -> Result<redis::Client, String> {
    open_client(valkey_url)
        .map_err(|err| format!("invalid valkey url for link_graph suggested_link store: {err}"))
}
/// `now_unix_f64` public function boundary for Wendao.
pub fn now_unix_f64() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |delta| delta.as_secs_f64())
}
/// `normalize_optional_string` public function boundary for Wendao.
pub fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let normalized = raw.trim().to_string();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized)
        }
    })
}
/// `suggestion_id_from_parts` public function boundary for Wendao.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn suggestion_id_from_parts(
    source_id: &str,
    target_id: &str,
    relation: &str,
    agent_id: &str,
    created_at_unix: f64,
) -> String {
    let raw = format!(
        "{source_id}|{target_id}|{relation}|{agent_id}|{:016x}",
        created_at_unix.to_bits()
    );
    format!("sl_{:016x}", xxh3_64(raw.as_bytes()))
}
/// `state_label` public function boundary for Wendao.
pub fn state_label(state: LinkGraphSuggestedLinkState) -> &'static str {
    match state {
        LinkGraphSuggestedLinkState::Provisional => "provisional",
        LinkGraphSuggestedLinkState::Promoted => "promoted",
        LinkGraphSuggestedLinkState::Rejected => "rejected",
    }
}
/// `push_stream_entry` public function boundary for Wendao.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn push_stream_entry(
    conn: &mut redis::Connection,
    stream_key: &str,
    payload: &str,
    max_entries: usize,
    ttl_seconds: Option<u64>,
    stream_label: &str,
) -> Result<(), String> {
    let ltrim_stop = i64::try_from(max_entries.max(1).saturating_sub(1)).unwrap_or(i64::MAX);
    redis::cmd("LPUSH")
        .arg(stream_key)
        .arg(payload)
        .query::<i64>(conn)
        .map_err(|err| format!("failed to LPUSH {stream_label} stream: {err}"))?;
    redis::cmd("LTRIM")
        .arg(stream_key)
        .arg(0)
        .arg(ltrim_stop)
        .query::<()>(conn)
        .map_err(|err| format!("failed to LTRIM {stream_label} stream: {err}"))?;
    if let Some(ttl) = ttl_seconds.filter(|value| *value > 0) {
        redis::cmd("EXPIRE")
            .arg(stream_key)
            .arg(ttl.cast_signed())
            .query::<i64>(conn)
            .map_err(|err| format!("failed to EXPIRE {stream_label} stream: {err}"))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/unit/link_graph/agentic/store/common.rs"]
mod tests;
