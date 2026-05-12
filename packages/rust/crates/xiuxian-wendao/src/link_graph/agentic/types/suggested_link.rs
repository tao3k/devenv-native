//! `link_graph::agentic::types::suggested_link` owns Wendao agentic types suggested link behavior.

use super::schema::LinkGraphSuggestedLinkState;
use serde::{Deserialize, Serialize};

/// Input payload for suggested-link state transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphSuggestedLinkDecisionRequest {
    /// Stable suggestion id to transition.
    pub suggestion_id: String,
    /// Next lifecycle state (`promoted` or `rejected`).
    pub target_state: LinkGraphSuggestedLinkState,
    /// Decision issuer id.
    pub decided_by: String,
    /// Human-readable decision reason.
    pub reason: String,
    /// Optional deterministic timestamp override for tests.
    #[serde(default)]
    pub decided_at_unix: Option<f64>,
}

/// Input payload for passive suggested-link logging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct LinkGraphSuggestedLinkRequest {
    /// Canonical source node id/path.
    pub source_id: String,
    /// Canonical target node id/path.
    pub target_id: String,
    /// Proposed relation label.
    pub relation: String,
    /// Proposal confidence in `[0.0, 1.0]`.
    #[serde(default)]
    pub confidence: f64,
    /// Human-readable bridge/evidence summary.
    pub evidence: String,
    /// Producer id (for example `qianhuan-architect`).
    pub agent_id: String,
    /// Optional deterministic timestamp override for tests.
    #[serde(default)]
    pub created_at_unix: Option<f64>,
}

/// Persisted suggested-link proposal record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct LinkGraphSuggestedLink {
    /// Stable suggestion identifier.
    #[serde(default)]
    pub suggestion_id: String,
    /// Persistence schema version.
    pub schema: String,
    /// Canonical source node id/path.
    pub source_id: String,
    /// Canonical target node id/path.
    pub target_id: String,
    /// Proposed relation label.
    pub relation: String,
    /// Proposal confidence in `[0.0, 1.0]`.
    pub confidence: f64,
    /// Human-readable bridge/evidence summary.
    pub evidence: String,
    /// Producer id (for example `qianhuan-architect`).
    pub agent_id: String,
    /// Proposal creation timestamp (unix seconds).
    pub created_at_unix: f64,
    /// Last state update timestamp (unix seconds).
    #[serde(default)]
    pub updated_at_unix: f64,
    /// Current promotion lifecycle state.
    pub promotion_state: LinkGraphSuggestedLinkState,
    /// Decision issuer for latest terminal transition.
    #[serde(default)]
    pub decision_by: Option<String>,
    /// Decision reason for latest terminal transition.
    #[serde(default)]
    pub decision_reason: Option<String>,
}

/// Persisted audit record for one suggested-link promotion/rejection decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct LinkGraphSuggestedLinkDecision {
    /// Persistence schema version.
    pub schema: String,
    /// Stable suggestion identifier.
    pub suggestion_id: String,
    /// Canonical source node id/path.
    pub source_id: String,
    /// Canonical target node id/path.
    pub target_id: String,
    /// Proposed relation label.
    pub relation: String,
    /// Previous lifecycle state before transition.
    pub previous_state: LinkGraphSuggestedLinkState,
    /// Target lifecycle state after transition.
    pub target_state: LinkGraphSuggestedLinkState,
    /// Decision issuer id.
    pub decided_by: String,
    /// Human-readable decision reason.
    pub reason: String,
    /// Decision timestamp (unix seconds).
    pub decided_at_unix: f64,
}

/// State transition result containing updated suggestion and decision audit row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphSuggestedLinkDecisionResult {
    /// Updated suggestion row after applying transition.
    pub suggestion: LinkGraphSuggestedLink,
    /// Decision audit record persisted for this transition.
    pub decision: LinkGraphSuggestedLinkDecision,
}
