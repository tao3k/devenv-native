use super::common::{push_stream_entry, redis_client, state_label};
use super::normalize::{normalize_decision_request, normalize_record_for_read};
use crate::link_graph::agentic::keys::{
    suggested_link_decision_stream_key, suggested_link_stream_key,
};
use crate::link_graph::agentic::types::{
    LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION, LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION,
    LinkGraphSuggestedLink, LinkGraphSuggestedLinkDecision, LinkGraphSuggestedLinkDecisionRequest,
    LinkGraphSuggestedLinkDecisionResult, LinkGraphSuggestedLinkState,
};
use crate::link_graph::runtime_config::{
    DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX, resolve_link_graph_agentic_runtime,
    resolve_link_graph_cache_runtime,
};

/// Apply one suggested-link decision transition (`provisional -> promoted/rejected`).
///
/// # Errors
///
/// Returns an error when runtime configuration cannot be resolved or the
/// decision cannot be persisted to Valkey.
pub fn valkey_suggested_link_decide(
    request: &LinkGraphSuggestedLinkDecisionRequest,
) -> Result<LinkGraphSuggestedLinkDecisionResult, String> {
    let cache_runtime = resolve_link_graph_cache_runtime()?;
    let agentic_runtime = resolve_link_graph_agentic_runtime();
    valkey_suggested_link_decide_with_valkey(
        request,
        &cache_runtime.valkey_url,
        Some(&cache_runtime.key_prefix),
        Some(agentic_runtime.suggested_link_max_entries),
        agentic_runtime.suggested_link_ttl_seconds,
    )
}

fn valkey_stop_index(limit: usize) -> Result<i64, String> {
    i64::try_from(limit.saturating_sub(1))
        .map_err(|_| format!("suggested_link decision limit exceeds Valkey LRANGE bounds: {limit}"))
}

/// Apply one suggested-link decision transition on explicit Valkey endpoint.
///
/// # Errors
///
/// Returns an error when the Valkey URL is invalid, the decision request cannot
/// be normalized or serialized, the decision target cannot be found, or the
/// Valkey write fails.
pub fn valkey_suggested_link_decide_with_valkey(
    request: &LinkGraphSuggestedLinkDecisionRequest,
    valkey_url: &str,
    key_prefix: Option<&str>,
    max_entries: Option<usize>,
    ttl_seconds: Option<u64>,
) -> Result<LinkGraphSuggestedLinkDecisionResult, String> {
    if valkey_url.trim().is_empty() {
        return Err("link_graph suggested_link valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);
    let bounded_max_entries = max_entries.unwrap_or(2000).max(1);
    let (suggestion_id, target_state, decided_by, reason, decided_at_unix) =
        normalize_decision_request(request)?;

    let stream_key = suggested_link_stream_key(prefix);
    let decision_stream_key = suggested_link_decision_stream_key(prefix);

    let client = redis_client(valkey_url)?;
    let mut conn = client.get_connection().map_err(|err| {
        format!("failed to connect valkey for link_graph suggested_link store: {err}")
    })?;
    let stop = valkey_stop_index(bounded_max_entries)?;

    let rows = redis::cmd("LRANGE")
        .arg(&stream_key)
        .arg(0)
        .arg(stop)
        .query::<Vec<String>>(&mut conn)
        .map_err(|err| format!("failed to LRANGE suggested_link stream: {err}"))?;

    let mut latest: Option<LinkGraphSuggestedLink> = None;
    for row in rows {
        let Ok(parsed) = serde_json::from_str::<LinkGraphSuggestedLink>(&row) else {
            continue;
        };
        if parsed.schema != LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION {
            continue;
        }
        let normalized = normalize_record_for_read(parsed);
        if normalized.suggestion_id == suggestion_id {
            latest = Some(normalized);
            break;
        }
    }

    let Some(previous) = latest else {
        return Err(format!(
            "suggested_link decision target not found for suggestion_id={suggestion_id}"
        ));
    };

    if previous.promotion_state != LinkGraphSuggestedLinkState::Provisional {
        return Err(format!(
            "suggested_link decision target already finalized: {}",
            state_label(previous.promotion_state)
        ));
    }

    let mut updated = previous.clone();
    updated.promotion_state = target_state;
    updated.updated_at_unix = decided_at_unix;
    updated.decision_by = Some(decided_by.clone());
    updated.decision_reason = Some(reason.clone());

    let decision = LinkGraphSuggestedLinkDecision {
        schema: LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION.to_string(),
        suggestion_id: suggestion_id.clone(),
        source_id: previous.source_id.clone(),
        target_id: previous.target_id.clone(),
        relation: previous.relation.clone(),
        previous_state: previous.promotion_state,
        target_state,
        decided_by,
        reason,
        decided_at_unix,
    };

    let updated_payload = serde_json::to_string(&updated)
        .map_err(|err| format!("failed to serialize updated suggested_link record: {err}"))?;
    let decision_payload = serde_json::to_string(&decision)
        .map_err(|err| format!("failed to serialize suggested_link decision record: {err}"))?;

    push_stream_entry(
        &mut conn,
        &stream_key,
        &updated_payload,
        bounded_max_entries,
        ttl_seconds,
        "suggested_link",
    )?;
    push_stream_entry(
        &mut conn,
        &decision_stream_key,
        &decision_payload,
        bounded_max_entries,
        ttl_seconds,
        "suggested_link_decision",
    )?;

    Ok(LinkGraphSuggestedLinkDecisionResult {
        suggestion: updated,
        decision,
    })
}

/// Read recent suggested-link decision audit rows.
///
/// # Errors
///
/// Returns an error when runtime configuration cannot be resolved or the Valkey
/// read fails.
pub fn valkey_suggested_link_decisions_recent(
    limit: usize,
) -> Result<Vec<LinkGraphSuggestedLinkDecision>, String> {
    let cache_runtime = resolve_link_graph_cache_runtime()?;
    valkey_suggested_link_decisions_recent_with_valkey(
        limit,
        &cache_runtime.valkey_url,
        Some(&cache_runtime.key_prefix),
    )
}

/// Read recent suggested-link decision audit rows from explicit Valkey endpoint.
///
/// # Errors
///
/// Returns an error when the Valkey URL is invalid, the limit cannot be
/// represented for `LRANGE`, or the read from Valkey fails.
pub fn valkey_suggested_link_decisions_recent_with_valkey(
    limit: usize,
    valkey_url: &str,
    key_prefix: Option<&str>,
) -> Result<Vec<LinkGraphSuggestedLinkDecision>, String> {
    if valkey_url.trim().is_empty() {
        return Err("link_graph suggested_link valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);
    let stream_key = suggested_link_decision_stream_key(prefix);
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
        .map_err(|err| format!("failed to LRANGE suggested_link decision stream: {err}"))?;

    let mut out: Vec<LinkGraphSuggestedLinkDecision> = Vec::new();
    for row in rows {
        let Ok(parsed) = serde_json::from_str::<LinkGraphSuggestedLinkDecision>(&row) else {
            continue;
        };
        if parsed.schema == LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION {
            out.push(parsed);
        }
    }
    Ok(out)
}
