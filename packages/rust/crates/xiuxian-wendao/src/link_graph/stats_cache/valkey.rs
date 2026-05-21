//! `link_graph::stats_cache::valkey` owns Wendao link graph stats cache valkey behavior.

use super::keys::stats_cache_key;
use super::payload::decode_stats_payload_if_fresh;
use super::runtime::{now_unix_f64, resolve_stats_cache_runtime};
use super::schema::LINK_GRAPH_STATS_CACHE_SCHEMA_VERSION;
use super::types::{LinkGraphStatsCachePayload, LinkGraphStatsCacheStats};
use crate::valkey_common::open_client;

/// Read `LinkGraph` stats cache payload from `Valkey` if still fresh.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
pub fn valkey_stats_cache_get(source_key: &str, ttl_sec: f64) -> Result<Option<String>, String> {
    let source_key = source_key.trim();
    if source_key.is_empty() || ttl_sec <= 0.0 {
        return Ok(None);
    }

    let (valkey_url, key_prefix) = resolve_stats_cache_runtime()?;
    let cache_key = stats_cache_key(source_key, &key_prefix);

    let client = redis_client(valkey_url.as_str())?;
    let mut conn = client
        .get_connection()
        .map_err(|e| format!("failed to connect valkey for link-graph stats cache: {e}"))?;
    let raw = redis::cmd("GET")
        .arg(&cache_key)
        .query::<Option<String>>(&mut conn)
        .map_err(|e| format!("failed to GET link-graph stats cache from valkey: {e}"))?;
    let Some(payload_raw) = raw else {
        return Ok(None);
    };

    let Some(valid_payload) = decode_stats_payload_if_fresh(&payload_raw, source_key, ttl_sec)
    else {
        let _ = redis::cmd("DEL").arg(&cache_key).query::<i64>(&mut conn);
        return Ok(None);
    };

    serde_json::to_string(&valid_payload)
        .map(Some)
        .map_err(|e| format!("failed to encode link-graph stats cache payload: {e}"))
}

/// Write `LinkGraph` stats cache payload to `Valkey` with TTL.
///
/// # Errors
///
/// Returns an error when payload parsing/serialization fails or Valkey operations fail.
pub fn valkey_stats_cache_set(
    source_key: &str,
    stats_json: &str,
    ttl_sec: f64,
) -> Result<(), String> {
    let source_key = source_key.trim();
    if source_key.is_empty() {
        return Err("source_key must be non-empty".to_string());
    }
    if ttl_sec <= 0.0 {
        return Ok(());
    }

    let parsed_stats = serde_json::from_str::<LinkGraphStatsCacheStats>(stats_json)
        .map_err(|e| format!("invalid link-graph stats payload: {e}"))?
        .normalize();
    let payload = LinkGraphStatsCachePayload {
        schema: LINK_GRAPH_STATS_CACHE_SCHEMA_VERSION.to_string(),
        source_key: source_key.to_string(),
        updated_at_unix: now_unix_f64(),
        stats: parsed_stats,
    };
    let encoded = serde_json::to_string(&payload)
        .map_err(|e| format!("failed to serialize link-graph stats cache payload: {e}"))?;

    let (valkey_url, key_prefix) = resolve_stats_cache_runtime()?;
    let cache_key = stats_cache_key(source_key, &key_prefix);
    let ttl_rounded = std::time::Duration::try_from_secs_f64(ttl_sec.ceil().max(1.0))
        .map_or(u64::MAX, |duration| duration.as_secs().max(1));

    let client = redis_client(valkey_url.as_str())?;
    let mut conn = client
        .get_connection()
        .map_err(|e| format!("failed to connect valkey for link-graph stats cache: {e}"))?;
    redis::cmd("SETEX")
        .arg(&cache_key)
        .arg(ttl_rounded)
        .arg(&encoded)
        .query::<()>(&mut conn)
        .map_err(|e| format!("failed to SETEX link-graph stats cache to valkey: {e}"))?;
    Ok(())
}

/// Delete `LinkGraph` stats cache entry for a source key.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
pub fn valkey_stats_cache_del(source_key: &str) -> Result<(), String> {
    let source_key = source_key.trim();
    if source_key.is_empty() {
        return Ok(());
    }
    let (valkey_url, key_prefix) = resolve_stats_cache_runtime()?;
    let cache_key = stats_cache_key(source_key, &key_prefix);

    let client = redis_client(valkey_url.as_str())?;
    let mut conn = client
        .get_connection()
        .map_err(|e| format!("failed to connect valkey for link-graph stats cache: {e}"))?;
    redis::cmd("DEL")
        .arg(&cache_key)
        .query::<i64>(&mut conn)
        .map_err(|e| format!("failed to DEL link-graph stats cache from valkey: {e}"))?;
    Ok(())
}

fn redis_client(valkey_url: &str) -> Result<redis::Client, String> {
    open_client(valkey_url)
        .map_err(|err| format!("invalid valkey url for link-graph stats cache: {err}"))
}

#[cfg(test)]
#[path = "../../../tests/unit/link_graph/stats_cache/valkey.rs"]
mod tests;
