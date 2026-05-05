use super::edge_updates::update_inbound_edge_scores;
use super::time::unix_seconds_to_f64;
use super::types::TouchUpdateSpec;
use crate::link_graph::saliency::store::read::load_current_state;
use crate::link_graph::saliency::{
    DEFAULT_DECAY_RATE, DEFAULT_SALIENCY_BASE, LINK_GRAPH_SALIENCY_SCHEMA_VERSION,
    LinkGraphSaliencyState, compute_link_graph_saliency, saliency_key,
};

pub(super) fn apply_touch_with_connection(
    conn: &mut redis::Connection,
    node_id: &str,
    key_prefix: &str,
    spec: TouchUpdateSpec,
) -> Result<LinkGraphSaliencyState, String> {
    let cache_key = saliency_key(node_id, key_prefix);
    let delta_activation = spec.activation_delta.max(1);
    let normalized_policy = spec.policy.normalized();
    let existing = load_current_state(conn, &cache_key, node_id, normalized_policy);

    // Settlement strategy:
    // - Use current score as the next baseline.
    // - Apply decay across elapsed time and only this touch delta for activation boost.
    // - Persist settled score as both `current_saliency` and next `saliency_base`.
    let (baseline, decay_rate, total_activation, delta_days) = if let Some(state) = existing {
        let elapsed_seconds =
            unix_seconds_to_f64((spec.now_unix - state.last_accessed_unix).max(0));
        (
            spec.saliency_base.unwrap_or(state.current_saliency),
            spec.decay_rate_override.unwrap_or(state.decay_rate),
            state.activation_count.saturating_add(delta_activation),
            elapsed_seconds / 86_400.0,
        )
    } else {
        (
            spec.saliency_base.unwrap_or(DEFAULT_SALIENCY_BASE),
            spec.decay_rate_override.unwrap_or(DEFAULT_DECAY_RATE),
            delta_activation,
            0.0,
        )
    };

    let settled_score = compute_link_graph_saliency(
        baseline,
        decay_rate,
        delta_activation,
        delta_days,
        normalized_policy,
    );
    let state = LinkGraphSaliencyState {
        schema: LINK_GRAPH_SALIENCY_SCHEMA_VERSION.to_string(),
        node_id: node_id.to_string(),
        saliency_base: settled_score,
        decay_rate,
        activation_count: total_activation,
        last_accessed_unix: spec.now_unix,
        current_saliency: settled_score,
        updated_at_unix: unix_seconds_to_f64(spec.now_unix),
    };
    let encoded = serde_json::to_string(&state)
        .map_err(|err| format!("failed to serialize link_graph saliency state: {err}"))?;

    redis::cmd("SET")
        .arg(&cache_key)
        .arg(encoded)
        .query::<()>(conn)
        .map_err(|err| format!("failed to SET link_graph saliency entry: {err}"))?;

    update_inbound_edge_scores(conn, node_id, key_prefix, settled_score);
    Ok(state)
}
