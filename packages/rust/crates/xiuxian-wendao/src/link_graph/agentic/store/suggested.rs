//! `link_graph::agentic::store::suggested` owns Wendao agentic store suggested behavior.

use std::collections::HashSet;

use crate::link_graph::runtime_config::{
    DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX, resolve_link_graph_agentic_runtime,
    resolve_link_graph_cache_runtime,
};

use super::common::{push_stream_entry, redis_client};
use super::normalize::{normalize_record_for_read, normalize_request};
use crate::link_graph::agentic::keys::suggested_link_stream_key;
use crate::link_graph::agentic::types::{
    LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION, LinkGraphSuggestedLink,
    LinkGraphSuggestedLinkRequest, LinkGraphSuggestedLinkState,
};

/// Append one suggested-link proposal to Valkey passive stream.
///
/// # Errors
///
/// Returns an error when runtime configuration cannot be resolved or the
/// request cannot be persisted to Valkey.
pub fn valkey_suggested_link_log(
    request: &LinkGraphSuggestedLinkRequest,
) -> Result<LinkGraphSuggestedLink, String> {
    let cache_runtime = resolve_link_graph_cache_runtime()?;
    let agentic_runtime = resolve_link_graph_agentic_runtime();
    valkey_suggested_link_log_with_valkey(
        request,
        &cache_runtime.valkey_url,
        Some(&cache_runtime.key_prefix),
        Some(agentic_runtime.suggested_link_max_entries),
        agentic_runtime.suggested_link_ttl_seconds,
    )
}

fn valkey_stop_index(limit: usize) -> Result<i64, String> {
    i64::try_from(limit.saturating_sub(1))
        .map_err(|_| format!("suggested_link limit exceeds Valkey LRANGE bounds: {limit}"))
}

/// Append one suggested-link proposal to explicit Valkey endpoint.
///
/// # Errors
///
/// Returns an error when the Valkey URL is invalid, the request cannot be
/// normalized or serialized, or the write to Valkey fails.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn valkey_suggested_link_log_with_valkey(
    request: &LinkGraphSuggestedLinkRequest,
    valkey_url: &str,
    key_prefix: Option<&str>,
    max_entries: Option<usize>,
    ttl_seconds: Option<u64>,
) -> Result<LinkGraphSuggestedLink, String> {
    if valkey_url.trim().is_empty() {
        return Err("link_graph suggested_link valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);
    let stream_key = suggested_link_stream_key(prefix);
    let bounded_max_entries = max_entries.unwrap_or(2000).max(1);
    let record = normalize_request(request)?;
    let payload = serde_json::to_string(&record)
        .map_err(|err| format!("failed to serialize suggested_link record: {err}"))?;

    let client = redis_client(valkey_url)?;
    let mut conn = client.get_connection().map_err(|err| {
        format!("failed to connect valkey for link_graph suggested_link store: {err}")
    })?;

    push_stream_entry(
        &mut conn,
        &stream_key,
        &payload,
        bounded_max_entries,
        ttl_seconds,
        "suggested_link",
    )?;

    Ok(record)
}

/// Read recent suggested-link proposals from Valkey passive stream.
///
/// # Errors
///
/// Returns an error when runtime configuration cannot be resolved or the Valkey
/// read fails.
pub fn valkey_suggested_link_recent(limit: usize) -> Result<Vec<LinkGraphSuggestedLink>, String> {
    let cache_runtime = resolve_link_graph_cache_runtime()?;
    valkey_suggested_link_recent_with_valkey(
        limit,
        &cache_runtime.valkey_url,
        Some(&cache_runtime.key_prefix),
    )
}

/// Read recent suggested-link proposals from explicit Valkey endpoint.
///
/// # Errors
///
/// Returns an error when the Valkey URL is invalid, the limit cannot be
/// represented for `LRANGE`, or the read from Valkey fails.
pub fn valkey_suggested_link_recent_with_valkey(
    limit: usize,
    valkey_url: &str,
    key_prefix: Option<&str>,
) -> Result<Vec<LinkGraphSuggestedLink>, String> {
    if valkey_url.trim().is_empty() {
        return Err("link_graph suggested_link valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);
    let stream_key = suggested_link_stream_key(prefix);
    let bounded_limit = limit.max(1);

    let client = redis_client(valkey_url)?;
    let mut conn = client.get_connection().map_err(|err| {
        format!("failed to connect valkey for link_graph suggested_link store: {err}")
    })?;
    let stop = valkey_stop_index(bounded_limit)?;

    let rows = redis::cmd("LRANGE")
        .arg(&stream_key)
        .arg(0)
        .arg(stop)
        .query::<Vec<String>>(&mut conn)
        .map_err(|err| format!("failed to LRANGE suggested_link stream: {err}"))?;

    let mut out: Vec<LinkGraphSuggestedLink> = Vec::new();
    for row in rows {
        if let Ok(parsed) = serde_json::from_str::<LinkGraphSuggestedLink>(&row)
            && parsed.schema == LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION
        {
            out.push(normalize_record_for_read(parsed));
        }
    }
    Ok(out)
}

/// Read recent suggested-link proposals as latest unique states.
///
/// # Errors
///
/// Returns an error when runtime configuration cannot be resolved or the Valkey
/// read fails.
pub fn valkey_suggested_link_recent_latest(
    limit: usize,
    state_filter: Option<LinkGraphSuggestedLinkState>,
) -> Result<Vec<LinkGraphSuggestedLink>, String> {
    let cache_runtime = resolve_link_graph_cache_runtime()?;
    let agentic_runtime = resolve_link_graph_agentic_runtime();
    valkey_suggested_link_recent_latest_with_valkey(
        limit,
        &cache_runtime.valkey_url,
        Some(&cache_runtime.key_prefix),
        state_filter,
        Some(agentic_runtime.suggested_link_max_entries),
    )
}

/// Read recent suggested-link proposals as latest unique states from explicit Valkey endpoint.
///
/// # Errors
///
/// Returns an error when the Valkey URL is invalid, the scan limit cannot be
/// represented for `LRANGE`, or the read from Valkey fails.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn valkey_suggested_link_recent_latest_with_valkey(
    limit: usize,
    valkey_url: &str,
    key_prefix: Option<&str>,
    state_filter: Option<LinkGraphSuggestedLinkState>,
    scan_limit: Option<usize>,
) -> Result<Vec<LinkGraphSuggestedLink>, String> {
    if valkey_url.trim().is_empty() {
        return Err("link_graph suggested_link valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);
    let stream_key = suggested_link_stream_key(prefix);
    let bounded_limit = limit.max(1);
    let bounded_scan_limit = scan_limit.unwrap_or(2000).max(bounded_limit);

    let client = redis_client(valkey_url)?;
    let mut conn = client.get_connection().map_err(|err| {
        format!("failed to connect valkey for link_graph suggested_link store: {err}")
    })?;
    let stop = valkey_stop_index(bounded_scan_limit)?;

    let rows = redis::cmd("LRANGE")
        .arg(&stream_key)
        .arg(0)
        .arg(stop)
        .query::<Vec<String>>(&mut conn)
        .map_err(|err| format!("failed to LRANGE suggested_link stream: {err}"))?;

    let mut out: Vec<LinkGraphSuggestedLink> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for row in rows {
        let Ok(parsed) = serde_json::from_str::<LinkGraphSuggestedLink>(&row) else {
            continue;
        };
        if parsed.schema != LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION {
            continue;
        }
        let normalized = normalize_record_for_read(parsed);
        if !seen.insert(normalized.suggestion_id.clone()) {
            continue;
        }
        if let Some(expected) = state_filter
            && normalized.promotion_state != expected
        {
            continue;
        }
        out.push(normalized);
        if out.len() >= bounded_limit {
            break;
        }
    }
    Ok(out)
}
