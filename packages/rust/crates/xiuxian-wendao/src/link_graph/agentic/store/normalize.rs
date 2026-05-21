//! Normalization and validation for agentic suggested-link records.

use super::common::{normalize_optional_string, now_unix_f64, suggestion_id_from_parts};
use crate::link_graph::agentic::types::{
    LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION, LinkGraphSuggestedLink,
    LinkGraphSuggestedLinkDecisionRequest, LinkGraphSuggestedLinkRequest,
    LinkGraphSuggestedLinkState,
};
/// `normalize_record_for_read` public function boundary for Wendao.
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

struct NormalizedSuggestedLinkRequest {
    source_id: String,
    target_id: String,
    relation: String,
    confidence: f64,
    evidence: String,
    agent_id: String,
    created_at_unix: f64,
}
/// `normalize_request` public function boundary for Wendao.
pub fn normalize_request(
    request: &LinkGraphSuggestedLinkRequest,
) -> Result<LinkGraphSuggestedLink, String> {
    let normalized = normalize_suggested_link_request(request)?;
    let suggestion_id = suggestion_id_from_parts(
        &normalized.source_id,
        &normalized.target_id,
        &normalized.relation,
        &normalized.agent_id,
        normalized.created_at_unix,
    );

    Ok(LinkGraphSuggestedLink {
        suggestion_id,
        schema: LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION.to_string(),
        source_id: normalized.source_id,
        target_id: normalized.target_id,
        relation: normalized.relation,
        confidence: normalized.confidence,
        evidence: normalized.evidence,
        agent_id: normalized.agent_id,
        created_at_unix: normalized.created_at_unix,
        updated_at_unix: normalized.created_at_unix,
        promotion_state: LinkGraphSuggestedLinkState::Provisional,
        decision_by: None,
        decision_reason: None,
    })
}

fn normalize_suggested_link_request(
    request: &LinkGraphSuggestedLinkRequest,
) -> Result<NormalizedSuggestedLinkRequest, String> {
    let created_at_unix = request.created_at_unix.unwrap_or_else(now_unix_f64);
    validate_unix_timestamp("created_at_unix", created_at_unix)?;
    Ok(NormalizedSuggestedLinkRequest {
        source_id: required_trimmed_field("source_id", &request.source_id)?,
        target_id: required_trimmed_field("target_id", &request.target_id)?,
        relation: required_trimmed_field("relation", &request.relation)?,
        confidence: request.confidence.clamp(0.0, 1.0),
        evidence: required_trimmed_field("evidence", &request.evidence)?,
        agent_id: required_trimmed_field("agent_id", &request.agent_id)?,
        created_at_unix,
    })
}
/// `normalize_decision_request` public function boundary for Wendao.
/// Tuple API boundary: this public API preserves byte or count pairs used by existing addressing contracts.
pub fn normalize_decision_request(
    request: &LinkGraphSuggestedLinkDecisionRequest,
) -> Result<(String, LinkGraphSuggestedLinkState, String, String, f64), String> {
    let suggestion_id = required_decision_field("suggestion_id", &request.suggestion_id)?;

    let target_state = request.target_state;
    if target_state == LinkGraphSuggestedLinkState::Provisional {
        return Err(
            "suggested_link decision target_state must be promoted or rejected".to_string(),
        );
    }

    let decided_by = required_decision_field("decided_by", &request.decided_by)?;
    let reason = required_decision_field("reason", &request.reason)?;

    let decided_at_unix = request.decided_at_unix.unwrap_or_else(now_unix_f64);
    validate_decision_unix_timestamp("decided_at_unix", decided_at_unix)?;

    Ok((
        suggestion_id,
        target_state,
        decided_by,
        reason,
        decided_at_unix,
    ))
}

fn required_trimmed_field(field: &str, value: &str) -> Result<String, String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(format!("suggested_link {field} must be non-empty"))
    } else {
        Ok(value)
    }
}

fn required_decision_field(field: &str, value: &str) -> Result<String, String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(format!("suggested_link decision {field} must be non-empty"))
    } else {
        Ok(value)
    }
}

fn validate_unix_timestamp(field: &str, value: f64) -> Result<(), String> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(format!(
            "suggested_link {field} must be finite and non-negative"
        ))
    }
}

fn validate_decision_unix_timestamp(field: &str, value: f64) -> Result<(), String> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(format!(
            "suggested_link decision {field} must be finite and non-negative"
        ))
    }
}
