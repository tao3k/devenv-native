//! `link_graph::agentic::types::schema` owns Wendao agentic types schema behavior.

use serde::{Deserialize, Serialize};

/// Schema version used by passive suggested-link proposal records.
pub const LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION: &str =
    "xiuxian_wendao.link_graph.suggested_link.v1";
/// Schema version used by suggested-link decision audit records.
pub const LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION: &str =
    "xiuxian_wendao.link_graph.suggested_link_decision.v1";

/// Lifecycle state for one suggested-link proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LinkGraphSuggestedLinkState {
    /// Proposal recorded but not promoted.
    #[default]
    Provisional,
    /// Proposal promoted to verified edge by gate.
    Promoted,
    /// Proposal rejected/obsoleted by gate.
    Rejected,
}
