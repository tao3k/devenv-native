//! `link_graph::agentic::idempotency` owns Wendao link graph agentic idempotency behavior.

use super::types::{LinkGraphSuggestedLink, LinkGraphSuggestedLinkRequest};

/// Build deterministic idempotency signature for one suggested-link identity.
#[must_use]
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn suggested_link_signature(
    source_id: &str,
    target_id: &str,
    relation: &str,
    agent_id: &str,
) -> String {
    format!(
        "{}|{}|{}|{}",
        source_id.trim(),
        target_id.trim(),
        relation.trim(),
        agent_id.trim()
    )
}

/// Build idempotency signature from one persistence request.
#[must_use]
pub fn suggested_link_signature_from_request(request: &LinkGraphSuggestedLinkRequest) -> String {
    suggested_link_signature(
        &request.source_id,
        &request.target_id,
        &request.relation,
        &request.agent_id,
    )
}

/// Build idempotency signature from one stored suggested-link row.
#[must_use]
pub fn suggested_link_signature_from_row(row: &LinkGraphSuggestedLink) -> String {
    suggested_link_signature(&row.source_id, &row.target_id, &row.relation, &row.agent_id)
}
