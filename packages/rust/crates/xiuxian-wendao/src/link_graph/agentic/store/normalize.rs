use super::common::{normalize_optional_string, now_unix_f64, suggestion_id_from_parts};
use crate::link_graph::agentic::types::{
    LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION, LinkGraphSuggestedLink,
    LinkGraphSuggestedLinkDecisionRequest, LinkGraphSuggestedLinkRequest,
    LinkGraphSuggestedLinkState,
};

pub fn normalize_record_for_read(mut record: LinkGraphSuggestedLink) -> LinkGraphSuggestedLink {
    if record.suggestion_id.trim().is_empty() {
        record.suggestion_id = suggestion_id_from_parts(
            &record.source_id,
            &record.target_id,
            &record.relation,
            &record.agent_id,
            record.created_at_unix,
        );
    }
    if !record.updated_at_unix.is_finite() || record.updated_at_unix <= 0.0 {
        record.updated_at_unix = record.created_at_unix;
    }
    record.decision_by = normalize_optional_string(record.decision_by);
    record.decision_reason = normalize_optional_string(record.decision_reason);
    record
}

pub fn normalize_request(
    request: &LinkGraphSuggestedLinkRequest,
) -> Result<LinkGraphSuggestedLink, String> {
    let source_id = request.source_id.trim().to_string();
    if source_id.is_empty() {
        return Err("suggested_link source_id must be non-empty".to_string());
    }

    let target_id = request.target_id.trim().to_string();
    if target_id.is_empty() {
        return Err("suggested_link target_id must be non-empty".to_string());
    }

    let relation = request.relation.trim().to_string();
    if relation.is_empty() {
        return Err("suggested_link relation must be non-empty".to_string());
    }

    let evidence = request.evidence.trim().to_string();
    if evidence.is_empty() {
        return Err("suggested_link evidence must be non-empty".to_string());
    }

    let agent_id = request.agent_id.trim().to_string();
    if agent_id.is_empty() {
        return Err("suggested_link agent_id must be non-empty".to_string());
    }

    let confidence = request.confidence.clamp(0.0, 1.0);
    let created_at_unix = request.created_at_unix.unwrap_or_else(now_unix_f64);
    if !created_at_unix.is_finite() || created_at_unix < 0.0 {
        return Err("suggested_link created_at_unix must be finite and non-negative".to_string());
    }
    let suggestion_id = suggestion_id_from_parts(
        &source_id,
        &target_id,
        &relation,
        &agent_id,
        created_at_unix,
    );

    Ok(LinkGraphSuggestedLink {
        suggestion_id,
        schema: LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION.to_string(),
        source_id,
        target_id,
        relation,
        confidence,
        evidence,
        agent_id,
        created_at_unix,
        updated_at_unix: created_at_unix,
        promotion_state: LinkGraphSuggestedLinkState::Provisional,
        decision_by: None,
        decision_reason: None,
    })
}

pub fn normalize_decision_request(
    request: &LinkGraphSuggestedLinkDecisionRequest,
) -> Result<(String, LinkGraphSuggestedLinkState, String, String, f64), String> {
    let suggestion_id = request.suggestion_id.trim().to_string();
    if suggestion_id.is_empty() {
        return Err("suggested_link decision suggestion_id must be non-empty".to_string());
    }

    let target_state = request.target_state;
    if target_state == LinkGraphSuggestedLinkState::Provisional {
        return Err(
            "suggested_link decision target_state must be promoted or rejected".to_string(),
        );
    }

    let decided_by = request.decided_by.trim().to_string();
    if decided_by.is_empty() {
        return Err("suggested_link decision decided_by must be non-empty".to_string());
    }

    let reason = request.reason.trim().to_string();
    if reason.is_empty() {
        return Err("suggested_link decision reason must be non-empty".to_string());
    }

    let decided_at_unix = request.decided_at_unix.unwrap_or_else(now_unix_f64);
    if !decided_at_unix.is_finite() || decided_at_unix < 0.0 {
        return Err(
            "suggested_link decision decided_at_unix must be finite and non-negative".to_string(),
        );
    }

    Ok((
        suggestion_id,
        target_state,
        decided_by,
        reason,
        decided_at_unix,
    ))
}
