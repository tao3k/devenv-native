//! Checkpoint envelope model.

use crate::runtime::BpmnInstanceState;

/// Current scaffold checkpoint format version.
pub const BPMN_CHECKPOINT_FORMAT_VERSION: u32 = 1;

/// Versioned checkpoint envelope for Valkey persistence.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCheckpointEnvelope {
    /// Checkpoint format version.
    pub version: u32,
    /// Monotonic checkpoint sequence.
    pub sequence: u64,
    /// Durable instance state payload.
    pub state: BpmnInstanceState,
}

impl BpmnCheckpointEnvelope {
    /// Creates a checkpoint envelope from one instance state.
    #[must_use]
    pub fn from_state(state: BpmnInstanceState) -> Self {
        Self {
            version: BPMN_CHECKPOINT_FORMAT_VERSION,
            sequence: state.sequence,
            state,
        }
    }
}
