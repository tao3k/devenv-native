//! Public join-progress runtime state.

use crate::ir_index_api::BpmnNodeIndex;

/// Join progress state for one BPMN join-like node.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JoinRuntimeState {
    /// Owning node index.
    pub node_index: BpmnNodeIndex,
    /// Optional activation id for structured inclusive joins.
    #[serde(default)]
    pub activation_id: Option<u64>,
    /// How many buffered arrivals are currently represented for the join.
    pub arrived: u32,
    /// How many arrivals are expected for completion.
    pub expected: u32,
    /// Per-incoming-edge buffered arrival counts aligned to the process
    /// incoming-edge order.
    #[serde(default)]
    pub incoming_counts: Vec<u32>,
}
