//! BPMN edge specification types.

use super::BpmnNodeIndex;
use std::sync::Arc;

/// Immutable BPMN edge specification.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnEdgeSpec {
    /// Source node index.
    pub from: BpmnNodeIndex,
    /// Destination node index.
    pub to: BpmnNodeIndex,
    /// Optional label used for conditional or named routing.
    pub label: Option<Arc<str>>,
}

impl BpmnEdgeSpec {
    /// Creates an edge specification.
    #[must_use]
    pub fn new(from: BpmnNodeIndex, to: BpmnNodeIndex, label: Option<impl AsRef<str>>) -> Self {
        Self {
            from,
            to,
            label: label.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }
}
