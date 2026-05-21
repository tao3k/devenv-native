//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::link_graph::runtime_config::{
    DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX, resolve_link_graph_cache_runtime,
};
use crate::link_graph::saliency::{
    LINK_GRAPH_SALIENCY_SCHEMA_VERSION, LinkGraphSaliencyPolicy, LinkGraphSaliencyState,
};
use crate::valkey_common::{normalize_key_prefix, open_client};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const VALKEY_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const VALKEY_IO_TIMEOUT: Duration = Duration::from_secs(5);

pub(in crate::link_graph::saliency::store) fn now_unix_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |delta| delta.as_secs().cast_signed())
}

pub(in crate::link_graph::saliency::store) fn now_unix_f64() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |delta| delta.as_secs_f64())
}

pub(in crate::link_graph::saliency::store) fn normalize_policy(
    alpha: Option<f64>,
    minimum: Option<f64>,
    maximum: Option<f64>,
) -> LinkGraphSaliencyPolicy {
    let mut policy = LinkGraphSaliencyPolicy::default();
    if let Some(alpha_value) = alpha {
        policy.alpha = alpha_value;
    }
    if let Some(minimum_value) = minimum {
        policy.minimum = minimum_value;
    }
    if let Some(maximum_value) = maximum {
        policy.maximum = maximum_value;
    }
    policy.normalized()
}

pub(in crate::link_graph::saliency::store) fn parse_saliency_payload(
    raw: &str,
    node_id: &str,
    policy: LinkGraphSaliencyPolicy,
) -> Option<LinkGraphSaliencyState> {
    let parsed = serde_json::from_str::<LinkGraphSaliencyState>(raw).ok()?;
    if parsed.node_id != node_id {
        return None;
    }
    repair_saliency_state(parsed, policy)
}

pub(in crate::link_graph::saliency::store) fn parse_saliency_payload_any_node(
    raw: &str,
    policy: LinkGraphSaliencyPolicy,
) -> Option<LinkGraphSaliencyState> {
    let parsed = serde_json::from_str::<LinkGraphSaliencyState>(raw).ok()?;
    repair_saliency_state(parsed, policy)
}

fn repair_saliency_state(
    parsed: LinkGraphSaliencyState,
    policy: LinkGraphSaliencyPolicy,
) -> Option<LinkGraphSaliencyState> {
    if parsed.schema != LINK_GRAPH_SALIENCY_SCHEMA_VERSION {
        return None;
    }
    if parsed.node_id.trim().is_empty() {
        return None;
    }

    let normalized = policy.normalized();
    let saliency = if parsed.current_saliency.is_finite() {
        parsed
            .current_saliency
            .clamp(normalized.minimum, normalized.maximum)
    } else {
        normalized.minimum
    };
    let mut repaired = parsed;
    repaired.current_saliency = saliency;
    if repaired.last_accessed_unix < 0 {
        repaired.last_accessed_unix = 0;
    }
    if repaired.updated_at_unix < 0.0 || !repaired.updated_at_unix.is_finite() {
        repaired.updated_at_unix = now_unix_f64();
    }
    Some(repaired)
}

pub(in crate::link_graph::saliency::store) fn redis_client(
    valkey_url: &str,
) -> Result<redis::Client, String> {
    open_client(valkey_url)
        .map_err(|err| format!("invalid valkey url for link_graph saliency store: {err}"))
}

pub(in crate::link_graph::saliency::store) fn redis_connection(
    valkey_url: &str,
) -> Result<redis::Connection, String> {
    let client = redis_client(valkey_url)?;
    let conn = client
        .get_connection_with_timeout(VALKEY_CONNECT_TIMEOUT)
        .map_err(|err| format!("failed to connect valkey for link_graph saliency store: {err}"))?;

    if let Err(err) = conn.set_read_timeout(Some(VALKEY_IO_TIMEOUT)) {
        log::warn!("failed to set valkey read timeout for link_graph saliency store: {err}");
    }
    if let Err(err) = conn.set_write_timeout(Some(VALKEY_IO_TIMEOUT)) {
        log::warn!("failed to set valkey write timeout for link_graph saliency store: {err}");
    }

    Ok(conn)
}

pub(in crate::link_graph::saliency::store) fn resolve_runtime() -> Result<(String, String), String>
{
    let runtime = resolve_link_graph_cache_runtime()?;
    let key_prefix = normalize_key_prefix(
        runtime.key_prefix.as_str(),
        DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX,
    );
    Ok((runtime.valkey_url, key_prefix))
}
