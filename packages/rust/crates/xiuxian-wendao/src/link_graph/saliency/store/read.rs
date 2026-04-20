use super::common::{parse_saliency_payload, redis_connection, resolve_runtime};
use crate::link_graph::runtime_config::DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX;
use crate::link_graph::saliency::{LinkGraphSaliencyPolicy, LinkGraphSaliencyState, saliency_key};
use std::collections::{HashMap, HashSet};

pub(super) const VALKEY_SALIENCY_MGET_BATCH_SIZE: usize = 512;

pub(in crate::link_graph::saliency::store) fn load_current_state(
    conn: &mut redis::Connection,
    cache_key: &str,
    node_id: &str,
    policy: LinkGraphSaliencyPolicy,
) -> Option<LinkGraphSaliencyState> {
    let raw = redis::cmd("GET")
        .arg(cache_key)
        .query::<Option<String>>(conn)
        .ok()?;
    let payload = raw?;
    let parsed = parse_saliency_payload(&payload, node_id, policy);
    if parsed.is_none() {
        let _ = redis::cmd("DEL").arg(cache_key).query::<i64>(conn);
    }
    parsed
}

/// Read one saliency state using runtime-configured Valkey settings.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
pub fn valkey_saliency_get(node_id: &str) -> Result<Option<LinkGraphSaliencyState>, String> {
    let (valkey_url, key_prefix) = resolve_runtime()?;
    valkey_saliency_get_with_valkey(node_id, &valkey_url, Some(&key_prefix))
}

/// Read one saliency state from a specific Valkey endpoint.
///
/// # Errors
///
/// Returns an error when inputs are invalid or Valkey operations fail.
pub fn valkey_saliency_get_with_valkey(
    node_id: &str,
    valkey_url: &str,
    key_prefix: Option<&str>,
) -> Result<Option<LinkGraphSaliencyState>, String> {
    let trimmed = node_id.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if valkey_url.trim().is_empty() {
        return Err("link_graph saliency valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);
    let cache_key = saliency_key(trimmed, prefix);

    let policy = LinkGraphSaliencyPolicy::default();
    let mut conn = redis_connection(valkey_url)?;

    let raw = redis::cmd("GET")
        .arg(&cache_key)
        .query::<Option<String>>(&mut conn)
        .map_err(|err| format!("failed to GET link_graph saliency entry: {err}"))?;
    let Some(payload_raw) = raw else {
        return Ok(None);
    };

    if let Some(state) = parse_saliency_payload(&payload_raw, trimmed, policy) {
        return Ok(Some(state));
    }

    let _ = redis::cmd("DEL").arg(&cache_key).query::<i64>(&mut conn);
    Ok(None)
}

/// Read many saliency states using runtime-configured Valkey settings.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
pub fn valkey_saliency_get_many(
    node_ids: &[String],
) -> Result<HashMap<String, LinkGraphSaliencyState>, String> {
    let (valkey_url, key_prefix) = resolve_runtime()?;
    valkey_saliency_get_many_with_valkey(node_ids, &valkey_url, Some(&key_prefix))
}

/// Read many saliency states from a specific Valkey endpoint.
///
/// Invalid or stale payloads are ignored and removed best-effort.
///
/// # Errors
///
/// Returns an error when inputs are invalid or Valkey operations fail.
pub fn valkey_saliency_get_many_with_valkey(
    node_ids: &[String],
    valkey_url: &str,
    key_prefix: Option<&str>,
) -> Result<HashMap<String, LinkGraphSaliencyState>, String> {
    if node_ids.is_empty() {
        return Ok(HashMap::new());
    }
    if valkey_url.trim().is_empty() {
        return Err("link_graph saliency valkey_url must be non-empty".to_string());
    }

    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);

    let mut ordered_node_ids: Vec<String> = Vec::new();
    let mut cache_keys: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for node_id in node_ids {
        let trimmed = node_id.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !seen.insert(trimmed.to_string()) {
            continue;
        }
        ordered_node_ids.push(trimmed.to_string());
        cache_keys.push(saliency_key(trimmed, prefix));
    }

    if ordered_node_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let policy = LinkGraphSaliencyPolicy::default();
    let mut conn = redis_connection(valkey_url)?;

    let mut states: HashMap<String, LinkGraphSaliencyState> = HashMap::new();
    let mut offset: usize = 0;
    for cache_chunk in cache_keys.chunks(VALKEY_SALIENCY_MGET_BATCH_SIZE) {
        let end = offset.saturating_add(cache_chunk.len());
        let payloads = redis::cmd("MGET")
            .arg(cache_chunk)
            .query::<Vec<Option<String>>>(&mut conn)
            .map_err(|err| format!("failed to MGET link_graph saliency entries: {err}"))?;

        for ((node_id, cache_key), payload) in ordered_node_ids[offset..end]
            .iter()
            .cloned()
            .zip(cache_chunk.iter())
            .zip(payloads)
        {
            let Some(payload_raw) = payload else {
                continue;
            };

            if let Some(state) = parse_saliency_payload(&payload_raw, node_id.as_str(), policy) {
                states.insert(node_id.clone(), state);
                continue;
            }

            let _ = redis::cmd("DEL").arg(cache_key).query::<i64>(&mut conn);
        }
        offset = end;
    }

    Ok(states)
}
