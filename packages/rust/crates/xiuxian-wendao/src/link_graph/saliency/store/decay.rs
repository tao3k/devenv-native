//! `link_graph::saliency::store::decay` owns Wendao saliency store decay behavior.

use super::common::{
    now_unix_i64, parse_saliency_payload_any_node, redis_connection, resolve_runtime,
};
use crate::link_graph::runtime_config::DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX;
use crate::link_graph::saliency::{
    LINK_GRAPH_SALIENCY_SCHEMA_VERSION, LinkGraphSaliencyDecaySweepRequest,
    LinkGraphSaliencyDecaySweepResult, LinkGraphSaliencyPolicy, LinkGraphSaliencyState,
    compute_link_graph_saliency,
};
use std::collections::HashSet;
use std::time::Duration;

const DECAY_SWEEP_SCAN_COUNT: usize = 512;

/// Run a global saliency decay sweep using runtime-configured Valkey settings.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
pub fn valkey_saliency_decay_all(
    request: LinkGraphSaliencyDecaySweepRequest,
) -> Result<LinkGraphSaliencyDecaySweepResult, String> {
    let (valkey_url, key_prefix) = resolve_runtime()?;
    valkey_saliency_decay_all_with_valkey(request, &valkey_url, Some(&key_prefix))
}

/// Run a global saliency decay sweep against an explicit Valkey endpoint.
///
/// # Errors
///
/// Returns an error when inputs are invalid or Valkey operations fail.
pub fn valkey_saliency_decay_all_with_valkey(
    request: LinkGraphSaliencyDecaySweepRequest,
    valkey_url: &str,
    key_prefix: Option<&str>,
) -> Result<LinkGraphSaliencyDecaySweepResult, String> {
    if valkey_url.trim().is_empty() {
        return Err("link_graph saliency valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);
    let now_unix = request.now_unix.unwrap_or_else(now_unix_i64);
    let policy = LinkGraphSaliencyPolicy::default();
    let mut conn = redis_connection(valkey_url)?;

    let keys = scan_saliency_keys(&mut conn, prefix)?;
    let mut result = LinkGraphSaliencyDecaySweepResult {
        now_unix,
        scanned_keys: keys.len(),
        updated_states: 0,
        deleted_states: 0,
    };

    for key in keys {
        let raw = redis::cmd("GET")
            .arg(&key)
            .query::<Option<String>>(&mut conn)
            .map_err(|err| {
                format!("failed to GET link_graph saliency entry during decay: {err}")
            })?;
        let Some(payload_raw) = raw else {
            continue;
        };

        let Some(state) = parse_saliency_payload_any_node(&payload_raw, policy) else {
            if redis::cmd("DEL").arg(&key).query::<i64>(&mut conn).is_ok() {
                result.deleted_states += 1;
            }
            continue;
        };

        if now_unix <= state.last_accessed_unix {
            continue;
        }

        let decayed_state = settle_decayed_state(state, policy, now_unix);
        let encoded = serde_json::to_string(&decayed_state)
            .map_err(|err| format!("failed to serialize link_graph saliency state: {err}"))?;
        redis::cmd("SET")
            .arg(&key)
            .arg(encoded)
            .query::<()>(&mut conn)
            .map_err(|err| {
                format!("failed to SET link_graph saliency entry during decay: {err}")
            })?;
        result.updated_states += 1;
    }

    Ok(result)
}

fn scan_saliency_keys(
    conn: &mut redis::Connection,
    key_prefix: &str,
) -> Result<Vec<String>, String> {
    let pattern = format!("{key_prefix}:saliency:*");
    let mut cursor: u64 = 0;
    let mut keys = Vec::new();
    let mut seen = HashSet::new();
    let mut seen_cursors: HashSet<u64> = HashSet::new();

    loop {
        if !seen_cursors.insert(cursor) {
            return Err(format!(
                "failed to SCAN link_graph saliency keys: repeated cursor {cursor}"
            ));
        }

        let (next_cursor, batch): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(&pattern)
            .arg("COUNT")
            .arg(DECAY_SWEEP_SCAN_COUNT)
            .query(conn)
            .map_err(|err| format!("failed to SCAN link_graph saliency keys: {err}"))?;
        for key in batch {
            if seen.insert(key.clone()) {
                keys.push(key);
            }
        }
        if next_cursor == 0 {
            break;
        }
        cursor = next_cursor;
    }

    keys.sort_unstable();
    Ok(keys)
}

fn settle_decayed_state(
    state: LinkGraphSaliencyState,
    policy: LinkGraphSaliencyPolicy,
    now_unix: i64,
) -> LinkGraphSaliencyState {
    let elapsed_seconds = unix_seconds_to_f64((now_unix - state.last_accessed_unix).max(0));
    let delta_days = elapsed_seconds / 86_400.0;
    let settled_score = compute_link_graph_saliency(
        state.current_saliency,
        state.decay_rate,
        0,
        delta_days,
        policy,
    );

    LinkGraphSaliencyState {
        schema: LINK_GRAPH_SALIENCY_SCHEMA_VERSION.to_string(),
        node_id: state.node_id,
        saliency_base: settled_score,
        decay_rate: state.decay_rate,
        activation_count: state.activation_count,
        last_accessed_unix: now_unix,
        current_saliency: settled_score,
        updated_at_unix: unix_seconds_to_f64(now_unix),
    }
}

fn unix_seconds_to_f64(seconds: i64) -> f64 {
    u64::try_from(seconds).map_or(0.0, |value| Duration::from_secs(value).as_secs_f64())
}
