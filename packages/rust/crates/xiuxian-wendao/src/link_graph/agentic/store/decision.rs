//! Suggested-link decision persistence and audit stream handling.

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

struct SuggestedLinkDecisionContext {
    suggestion_id: String,
    target_state: LinkGraphSuggestedLinkState,
    decided_by: String,
    reason: String,
    decided_at_unix: f64,
    stream_key: String,
    decision_stream_key: String,
    bounded_max_entries: usize,
    ttl_seconds: Option<u64>,
}

struct SuggestedLinkDecisionPayloads {
    updated: String,
    decision: String,
}

/// Apply one suggested-link decision transition on explicit Valkey endpoint.
///
/// # Errors
///
/// Returns an error when the Valkey URL is invalid, the decision request cannot
/// be normalized or serialized, the decision target cannot be found, or the
/// Valkey write fails.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn valkey_suggested_link_decide_with_valkey(
    request: &LinkGraphSuggestedLinkDecisionRequest,
    valkey_url: &str,
    key_prefix: Option<&str>,
    max_entries: Option<usize>,
    ttl_seconds: Option<u64>,
) -> Result<LinkGraphSuggestedLinkDecisionResult, String> {
    validate_valkey_url(valkey_url)?;
    let context = suggested_link_decision_context(request, key_prefix, max_entries, ttl_seconds)?;
    let client = redis_client(valkey_url.trim())?;
    let mut conn = client.get_connection().map_err(|err| {
        format!("failed to connect valkey for link_graph suggested_link store: {err}")
    })?;

    let rows = read_suggested_link_rows(&mut conn, &context)?;
    let previous = find_suggested_link_decision_target(rows, context.suggestion_id.as_str())?;
    ensure_suggested_link_is_provisional(&previous)?;

    let updated = updated_suggested_link(previous.clone(), &context);
    let decision = suggested_link_decision_record(&previous, &context);
    let payloads = suggested_link_decision_payloads(&updated, &decision)?;
    persist_suggested_link_decision(&mut conn, &context, &payloads)?;

    Ok(LinkGraphSuggestedLinkDecisionResult {
        suggestion: updated,
        decision,
    })
}

fn validate_valkey_url(valkey_url: &str) -> Result<(), String> {
    if valkey_url.trim().is_empty() {
        Err("link_graph suggested_link valkey_url must be non-empty".to_string())
    } else {
        Ok(())
    }
}

fn suggested_link_decision_context(
    request: &LinkGraphSuggestedLinkDecisionRequest,
    key_prefix: Option<&str>,
    max_entries: Option<usize>,
    ttl_seconds: Option<u64>,
) -> Result<SuggestedLinkDecisionContext, String> {
    let prefix = key_prefix
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX);
    let (suggestion_id, target_state, decided_by, reason, decided_at_unix) =
        normalize_decision_request(request)?;
    Ok(SuggestedLinkDecisionContext {
        suggestion_id,
        target_state,
        decided_by,
        reason,
        decided_at_unix,
        stream_key: suggested_link_stream_key(prefix),
        decision_stream_key: suggested_link_decision_stream_key(prefix),
        bounded_max_entries: max_entries.unwrap_or(2000).max(1),
        ttl_seconds,
    })
}

fn read_suggested_link_rows(
    conn: &mut redis::Connection,
    context: &SuggestedLinkDecisionContext,
) -> Result<Vec<String>, String> {
    let stop = valkey_stop_index(context.bounded_max_entries)?;
    redis::cmd("LRANGE")
        .arg(&context.stream_key)
        .arg(0)
        .arg(stop)
        .query::<Vec<String>>(conn)
        .map_err(|err| format!("failed to LRANGE suggested_link stream: {err}"))
}

fn find_suggested_link_decision_target(
    rows: Vec<String>,
    suggestion_id: &str,
) -> Result<LinkGraphSuggestedLink, String> {
    rows.into_iter()
        .filter_map(|row| parse_suggested_link_row(&row))
        .find(|record| record.suggestion_id == suggestion_id)
        .ok_or_else(|| {
            format!("suggested_link decision target not found for suggestion_id={suggestion_id}")
        })
}

fn parse_suggested_link_row(row: &str) -> Option<LinkGraphSuggestedLink> {
    let parsed = serde_json::from_str::<LinkGraphSuggestedLink>(row).ok()?;
    (parsed.schema == LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION)
        .then(|| normalize_record_for_read(parsed))
}

fn ensure_suggested_link_is_provisional(previous: &LinkGraphSuggestedLink) -> Result<(), String> {
    if previous.promotion_state == LinkGraphSuggestedLinkState::Provisional {
        Ok(())
    } else {
        Err(format!(
            "suggested_link decision target already finalized: {}",
            state_label(previous.promotion_state)
        ))
    }
}

fn updated_suggested_link(
    previous: LinkGraphSuggestedLink,
    context: &SuggestedLinkDecisionContext,
) -> LinkGraphSuggestedLink {
    LinkGraphSuggestedLink {
        promotion_state: context.target_state,
        updated_at_unix: context.decided_at_unix,
        decision_by: Some(context.decided_by.clone()),
        decision_reason: Some(context.reason.clone()),
        ..previous
    }
}

fn suggested_link_decision_record(
    previous: &LinkGraphSuggestedLink,
    context: &SuggestedLinkDecisionContext,
) -> LinkGraphSuggestedLinkDecision {
    LinkGraphSuggestedLinkDecision {
        schema: LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION.to_string(),
        suggestion_id: context.suggestion_id.clone(),
        source_id: previous.source_id.clone(),
        target_id: previous.target_id.clone(),
        relation: previous.relation.clone(),
        previous_state: previous.promotion_state,
        target_state: context.target_state,
        decided_by: context.decided_by.clone(),
        reason: context.reason.clone(),
        decided_at_unix: context.decided_at_unix,
    }
}

fn suggested_link_decision_payloads(
    updated: &LinkGraphSuggestedLink,
    decision: &LinkGraphSuggestedLinkDecision,
) -> Result<SuggestedLinkDecisionPayloads, String> {
    Ok(SuggestedLinkDecisionPayloads {
        updated: serde_json::to_string(updated)
            .map_err(|err| format!("failed to serialize updated suggested_link record: {err}"))?,
        decision: serde_json::to_string(decision)
            .map_err(|err| format!("failed to serialize suggested_link decision record: {err}"))?,
    })
}

fn persist_suggested_link_decision(
    conn: &mut redis::Connection,
    context: &SuggestedLinkDecisionContext,
    payloads: &SuggestedLinkDecisionPayloads,
) -> Result<(), String> {
    push_stream_entry(
        conn,
        &context.stream_key,
        &payloads.updated,
        context.bounded_max_entries,
        context.ttl_seconds,
        "suggested_link",
    )?;
    push_stream_entry(
        conn,
        &context.decision_stream_key,
        &payloads.decision,
        context.bounded_max_entries,
        context.ttl_seconds,
        "suggested_link_decision",
    )
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
