use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::runtime_config::{
    LinkGraphCacheRuntimeConfig, resolve_link_graph_cache_runtime,
};
use crate::link_graph::saliency::{
    DEFAULT_SALIENCY_BASE, LINK_GRAPH_SALIENCY_SCHEMA_VERSION, LinkGraphSaliencyPolicy,
    LinkGraphSaliencyState, compute_link_graph_saliency, edge_in_key, edge_out_key, saliency_key,
};
use crate::valkey_common::open_client;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn redis_client(valkey_url: &str) -> Result<redis::Client, String> {
    open_client(valkey_url)
        .map_err(|err| format!("invalid valkey url for link-graph graphmem sync: {err}"))
}

pub(super) fn sync_graphmem_state_to_valkey(
    index: &LinkGraphIndex,
    runtime: &LinkGraphCacheRuntimeConfig,
) -> Result<(), String> {
    let client = redis_client(runtime.valkey_url.as_str())?;
    let mut conn = client
        .get_connection()
        .map_err(|e| format!("failed to connect valkey for link-graph graphmem sync: {e}"))?;

    let now_delta = SystemTime::now().duration_since(UNIX_EPOCH).ok();
    let now_unix = now_delta
        .as_ref()
        .map_or(0, |delta| delta.as_secs().cast_signed());
    let now_unix_f64 = now_delta
        .as_ref()
        .map_or(0.0, std::time::Duration::as_secs_f64);
    let policy = LinkGraphSaliencyPolicy::default();

    let mut score_by_doc: HashMap<String, f64> = HashMap::with_capacity(index.docs_by_id.len());
    for doc in index.docs_by_id.values() {
        let node_id = doc.id.as_str();
        let state_key = saliency_key(node_id, &runtime.key_prefix);
        let existing_raw = redis::cmd("GET")
            .arg(&state_key)
            .query::<Option<String>>(&mut conn)
            .map_err(|e| format!("failed to GET saliency seed for '{node_id}': {e}"))?;

        let existing_score = existing_raw
            .as_deref()
            .and_then(|raw| serde_json::from_str::<LinkGraphSaliencyState>(raw).ok())
            .filter(|state| {
                state.schema == LINK_GRAPH_SALIENCY_SCHEMA_VERSION && state.node_id == node_id
            })
            .map(|state| state.current_saliency);

        if let Some(score) = existing_score {
            score_by_doc.insert(node_id.to_string(), score);
            continue;
        }

        let seeded_score =
            compute_link_graph_saliency(doc.saliency_base, doc.decay_rate, 0, 0.0, policy);
        let seeded_state = LinkGraphSaliencyState {
            schema: LINK_GRAPH_SALIENCY_SCHEMA_VERSION.to_string(),
            node_id: node_id.to_string(),
            saliency_base: seeded_score,
            decay_rate: doc.decay_rate,
            activation_count: 0,
            last_accessed_unix: now_unix,
            current_saliency: seeded_score,
            updated_at_unix: now_unix_f64,
        };
        let encoded = serde_json::to_string(&seeded_state)
            .map_err(|e| format!("failed to serialize seeded saliency for '{node_id}': {e}"))?;
        redis::cmd("SET")
            .arg(&state_key)
            .arg(encoded)
            .query::<()>(&mut conn)
            .map_err(|e| format!("failed to SET seeded saliency for '{node_id}': {e}"))?;
        score_by_doc.insert(node_id.to_string(), seeded_score);
    }

    let in_pattern = format!("{}:kg:edge:in:*", runtime.key_prefix);
    let out_pattern = format!("{}:kg:edge:out:*", runtime.key_prefix);
    let stale_in_keys = redis::cmd("KEYS")
        .arg(&in_pattern)
        .query::<Vec<String>>(&mut conn)
        .unwrap_or_default();
    if !stale_in_keys.is_empty() {
        let _ = redis::cmd("DEL").arg(stale_in_keys).query::<i64>(&mut conn);
    }
    let stale_out_keys = redis::cmd("KEYS")
        .arg(&out_pattern)
        .query::<Vec<String>>(&mut conn)
        .unwrap_or_default();
    if !stale_out_keys.is_empty() {
        let _ = redis::cmd("DEL")
            .arg(stale_out_keys)
            .query::<i64>(&mut conn);
    }

    for (from, targets) in &index.outgoing {
        let out_key = edge_out_key(from, &runtime.key_prefix);
        for to in targets {
            let in_key = edge_in_key(to, &runtime.key_prefix);
            let _ = redis::cmd("SADD")
                .arg(&in_key)
                .arg(from)
                .query::<i64>(&mut conn);
            let score = score_by_doc
                .get(to)
                .copied()
                .unwrap_or(DEFAULT_SALIENCY_BASE);
            let _ = redis::cmd("ZADD")
                .arg(&out_key)
                .arg(score)
                .arg(to)
                .query::<i64>(&mut conn);
        }
    }

    Ok(())
}

pub(super) fn sync_graphmem_state_best_effort(index: &LinkGraphIndex) {
    let Ok(runtime) = resolve_link_graph_cache_runtime() else {
        return;
    };
    let _ = sync_graphmem_state_to_valkey(index, &runtime);
}

#[cfg(test)]
#[path = "../../../../tests/unit/link_graph/index/build/graphmem.rs"]
mod tests;
