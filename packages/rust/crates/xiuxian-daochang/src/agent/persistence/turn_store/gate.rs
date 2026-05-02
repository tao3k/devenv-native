use xiuxian_memory_engine::{
    Episode, EpisodeStore, MemoryGateDecision, MemoryGateEvent, MemoryGatePolicy,
    MemoryGateVerdict, MemoryUtilityLedger,
};

use crate::agent::Agent;
use crate::observability::SessionEvent;

use super::types::{StoredTurnEpisode, TurnStoreOutcome};

impl Agent {
    pub(super) async fn evaluate_turn_memory_gate(
        &self,
        store: &EpisodeStore,
        session_id: &str,
        stored: &StoredTurnEpisode,
        tool_count: u32,
        outcome: &TurnStoreOutcome,
        gate_policy: MemoryGatePolicy,
    ) {
        let Some(stored_episode) = store.get(&stored.id) else {
            return;
        };
        let ledger = build_turn_memory_ledger(&stored_episode, tool_count, outcome.reward);
        let decision = gate_policy.evaluate(
            &ledger,
            vec![
                format!("react:tool_calls:{tool_count}"),
                format!("react:outcome:{}", outcome.label),
            ],
            vec![format!("graph:turn_tool_count:{tool_count}")],
            vec![format!("omega:reward={:.3}", outcome.reward)],
        );
        let gate_event = MemoryGateEvent::from_decision(
            session_id,
            Self::next_runtime_turn_id(),
            &stored.id,
            &ledger,
            decision.clone(),
        );

        log_turn_memory_gate_decision(session_id, stored, &decision, &gate_event, &ledger);
        Self::maybe_purge_obsolete_episode(store, session_id, stored, &decision);
        self.publish_memory_stream_event(memory_gate_event_fields(
            session_id,
            stored,
            &decision,
            &gate_event,
        ))
        .await;
        if matches!(decision.verdict, MemoryGateVerdict::Promote) {
            log_turn_memory_promoted(session_id, stored, &decision);
            self.publish_memory_stream_event(memory_promoted_event_fields(
                session_id,
                stored,
                &decision,
                &gate_event,
                &ledger,
                &stored_episode,
            ))
            .await;
        }
    }

    fn maybe_purge_obsolete_episode(
        store: &EpisodeStore,
        session_id: &str,
        stored: &StoredTurnEpisode,
        decision: &MemoryGateDecision,
    ) {
        if matches!(decision.verdict, MemoryGateVerdict::Obsolete)
            && store.delete_episode(&stored.id)
        {
            tracing::debug!(
                event = SessionEvent::MemoryGateEvaluated.as_str(),
                session_id,
                episode_id = %stored.id,
                episode_source = stored.source,
                action = "purged",
                "memory episode purged by gate decision"
            );
        }
    }
}

fn build_turn_memory_ledger(
    stored_episode: &Episode,
    tool_count: u32,
    reward: f32,
) -> MemoryUtilityLedger {
    let normalized_tool_count = u8::try_from(tool_count.min(6)).unwrap_or(6);
    let tool_count_f32 = f32::from(normalized_tool_count);
    let react_score = if reward > 0.0 {
        (0.72 + (tool_count_f32 * 0.04)).clamp(0.0, 1.0)
    } else {
        (0.20 + (tool_count_f32 * 0.01)).clamp(0.0, 1.0)
    };
    let graph_score = if tool_count > 0 { 0.64 } else { 0.45 };
    let omega_score = if reward > 0.0 { 0.78 } else { 0.22 };
    MemoryUtilityLedger::from_episode(stored_episode, react_score, graph_score, omega_score)
}

fn log_turn_memory_gate_decision(
    session_id: &str,
    stored: &StoredTurnEpisode,
    decision: &MemoryGateDecision,
    gate_event: &MemoryGateEvent,
    ledger: &MemoryUtilityLedger,
) {
    tracing::debug!(
        event = SessionEvent::MemoryGateEvaluated.as_str(),
        session_id,
        episode_id = %stored.id,
        episode_source = stored.source,
        verdict = decision.verdict.as_str(),
        confidence = decision.confidence,
        ttl_score = gate_event.ttl_score,
        utility_score = ledger.utility_score,
        react_evidence_count = decision.react_evidence_refs.len(),
        graph_evidence_count = decision.graph_evidence_refs.len(),
        omega_factor_count = decision.omega_factors.len(),
        react_evidence_refs = ?decision.react_evidence_refs,
        graph_evidence_refs = ?decision.graph_evidence_refs,
        omega_factors = ?decision.omega_factors,
        next_action = %decision.next_action,
        reason = %decision.reason,
        "memory gate decision evaluated"
    );
}

fn log_turn_memory_promoted(
    session_id: &str,
    stored: &StoredTurnEpisode,
    decision: &MemoryGateDecision,
) {
    tracing::info!(
        event = SessionEvent::MemoryPromoted.as_str(),
        session_id,
        episode_id = %stored.id,
        episode_source = stored.source,
        confidence = decision.confidence,
        next_action = %decision.next_action,
        reason = %decision.reason,
        "memory episode promoted and queued for durable knowledge ingestion"
    );
}

fn memory_gate_event_fields(
    session_id: &str,
    stored: &StoredTurnEpisode,
    decision: &MemoryGateDecision,
    gate_event: &MemoryGateEvent,
) -> Vec<(String, String)> {
    vec![
        ("kind".to_string(), "memory_gate_event".to_string()),
        ("session_id".to_string(), session_id.to_string()),
        ("episode_id".to_string(), stored.id.clone()),
        ("episode_source".to_string(), stored.source.to_string()),
        ("turn_id".to_string(), gate_event.turn_id.to_string()),
        (
            "state_before".to_string(),
            gate_event.state_before.as_str().to_string(),
        ),
        (
            "state_after".to_string(),
            gate_event.state_after.as_str().to_string(),
        ),
        (
            "ttl_score".to_string(),
            format!("{:.3}", gate_event.ttl_score),
        ),
        ("verdict".to_string(), decision.verdict.as_str().to_string()),
        (
            "confidence".to_string(),
            format!("{:.3}", decision.confidence),
        ),
        (
            "react_evidence_count".to_string(),
            decision.react_evidence_refs.len().to_string(),
        ),
        (
            "graph_evidence_count".to_string(),
            decision.graph_evidence_refs.len().to_string(),
        ),
        (
            "omega_factor_count".to_string(),
            decision.omega_factors.len().to_string(),
        ),
        (
            "react_evidence_refs".to_string(),
            encode_string_list_for_stream(&decision.react_evidence_refs),
        ),
        (
            "graph_evidence_refs".to_string(),
            encode_string_list_for_stream(&decision.graph_evidence_refs),
        ),
        (
            "omega_factors".to_string(),
            encode_string_list_for_stream(&decision.omega_factors),
        ),
        ("next_action".to_string(), decision.next_action.clone()),
    ]
}

fn memory_promoted_event_fields(
    session_id: &str,
    stored: &StoredTurnEpisode,
    decision: &MemoryGateDecision,
    gate_event: &MemoryGateEvent,
    ledger: &MemoryUtilityLedger,
    stored_episode: &Episode,
) -> Vec<(String, String)> {
    vec![
        ("kind".to_string(), "memory_promoted".to_string()),
        ("session_id".to_string(), session_id.to_string()),
        ("episode_id".to_string(), stored.id.clone()),
        ("episode_source".to_string(), stored.source.to_string()),
        (
            "scope_key".to_string(),
            stored_episode.scope_key().to_string(),
        ),
        ("turn_id".to_string(), gate_event.turn_id.to_string()),
        ("verdict".to_string(), decision.verdict.as_str().to_string()),
        (
            "confidence".to_string(),
            format!("{:.3}", decision.confidence),
        ),
        (
            "utility_score".to_string(),
            format!("{:.3}", ledger.utility_score),
        ),
        ("ttl_score".to_string(), format!("{:.3}", ledger.ttl_score)),
        ("q_value".to_string(), format!("{:.3}", ledger.q_value)),
        (
            "failure_rate".to_string(),
            format!("{:.3}", ledger.failure_rate),
        ),
        ("usage_count".to_string(), ledger.usage_count.to_string()),
        (
            "intent_excerpt".to_string(),
            stream_excerpt(&stored_episode.intent, 512),
        ),
        (
            "experience_excerpt".to_string(),
            stream_excerpt(&stored_episode.experience, 1024),
        ),
        ("outcome".to_string(), stored_episode.outcome.clone()),
        ("reason".to_string(), stream_excerpt(&decision.reason, 512)),
        (
            "react_evidence_refs".to_string(),
            encode_string_list_for_stream(&decision.react_evidence_refs),
        ),
        (
            "graph_evidence_refs".to_string(),
            encode_string_list_for_stream(&decision.graph_evidence_refs),
        ),
        (
            "omega_factors".to_string(),
            encode_string_list_for_stream(&decision.omega_factors),
        ),
        ("next_action".to_string(), decision.next_action.clone()),
        (
            "knowledge_ingest_hint".to_string(),
            "knowledge.ingest_candidate".to_string(),
        ),
    ]
}

fn encode_string_list_for_stream(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string())
}

fn stream_excerpt(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let trimmed = input.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let keep = max_chars.saturating_sub(3);
    let mut out = trimmed.chars().take(keep).collect::<String>();
    out.push_str("...");
    out
}
