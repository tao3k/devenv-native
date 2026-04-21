//! Public token-frontier runtime state.

use crate::ir_index_api::BpmnNodeIndex;

/// Structured inclusive-join activation metadata carried by a token.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InclusiveJoinHint {
    /// Stable activation id shared by all branches from one inclusive split.
    pub activation_id: u64,
    /// Matching structured inclusive-join node index.
    pub join_node_index: BpmnNodeIndex,
    /// Number of branch arrivals required before the join may activate.
    pub expected_arrivals: u32,
}

/// Runtime token record for the scaffold state model.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TokenRecord {
    /// Monotonic token identifier within an instance.
    pub token_id: u64,
    /// Current node position.
    pub node_index: BpmnNodeIndex,
    /// Incoming sequence-flow edge index that routed this token to the node.
    #[serde(default)]
    pub incoming_edge_index: Option<u32>,
    /// Optional structured inclusive-join activation metadata.
    #[serde(default)]
    pub inclusive_join_hint: Option<InclusiveJoinHint>,
}
