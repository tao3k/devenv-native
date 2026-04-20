//! Public token-frontier runtime state.

use crate::ir_index_api::BpmnNodeIndex;

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
}
