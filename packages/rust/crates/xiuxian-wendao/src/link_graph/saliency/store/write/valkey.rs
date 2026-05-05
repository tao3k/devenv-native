use super::coactivation::propagate_coactivation;
use super::touch::apply_touch_with_connection;
use super::types::TouchUpdateSpec;
use crate::link_graph::runtime_config::DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX;
use crate::link_graph::saliency::store::common::{
    normalize_policy, now_unix_i64, redis_connection, resolve_runtime,
};
use crate::link_graph::saliency::{
    LinkGraphSaliencyState, LinkGraphSaliencyTouchRequest, saliency_key,
};

/// Delete one saliency state entry by node id.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
pub fn valkey_saliency_del(node_id: &str) -> Result<(), String> {
    let (valkey_url, key_prefix) = resolve_runtime()?;
    let trimmed = node_id.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let cache_key = saliency_key(trimmed, &key_prefix);
    let mut conn = redis_connection(&valkey_url)?;
    redis::cmd("DEL")
        .arg(&cache_key)
        .query::<i64>(&mut conn)
        .map_err(|err| format!("failed to DEL link_graph saliency entry: {err}"))?;
    Ok(())
}

/// Update saliency using runtime-configured Valkey settings.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
pub fn valkey_saliency_touch(
    request: LinkGraphSaliencyTouchRequest,
) -> Result<LinkGraphSaliencyState, String> {
    let (valkey_url, key_prefix) = resolve_runtime()?;
    valkey_saliency_touch_with_valkey(request, &valkey_url, Some(&key_prefix))
}

/// Update saliency using an explicit Valkey endpoint.
///
/// This applies decay + activation settlement and persists the resulting state.
///
/// # Errors
///
/// Returns an error when inputs are invalid, serialization fails, or Valkey operations fail.
pub fn valkey_saliency_touch_with_valkey(
    request: LinkGraphSaliencyTouchRequest,
    valkey_url: &str,
    key_prefix: Option<&str>,
) -> Result<LinkGraphSaliencyState, String> {
    let LinkGraphSaliencyTouchRequest {
        node_id,
        activation_delta,
        saliency_base,
        decay_rate: decay_rate_override,
        alpha,
        minimum_saliency,
        maximum_saliency,
        now_unix,
    } = request;

    let node_id = node_id.trim();
    if node_id.is_empty() {
        return Err("node_id must be non-empty".to_string());
    }
    if valkey_url.trim().is_empty() {
        return Err("link_graph saliency valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);
    let now_unix = now_unix.unwrap_or_else(now_unix_i64);

    let policy = normalize_policy(alpha, minimum_saliency, maximum_saliency);
    let mut conn = redis_connection(valkey_url)?;

    let state = apply_touch_with_connection(
        &mut conn,
        node_id,
        prefix,
        TouchUpdateSpec {
            activation_delta,
            saliency_base,
            decay_rate_override,
            policy,
            now_unix,
        },
    )?;
    propagate_coactivation(&mut conn, node_id, prefix, now_unix, policy);
    Ok(state)
}
