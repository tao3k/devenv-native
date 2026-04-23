//! Public BPMN edge contract owner.

use crate::ir_index_api::BpmnNodeIndex;
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
    /// Optional bounded condition expression attached to this sequence flow.
    #[serde(default)]
    pub condition_expression: Option<Arc<str>>,
}

impl BpmnEdgeSpec {
    /// Creates an edge specification.
    #[must_use]
    pub fn new(from: BpmnNodeIndex, to: BpmnNodeIndex, label: Option<impl AsRef<str>>) -> Self {
        Self {
            from,
            to,
            label: label.map(|value| Arc::<str>::from(value.as_ref())),
            condition_expression: None,
        }
    }

    /// Attaches one bounded condition expression to the edge.
    #[must_use]
    pub fn with_condition_expression(mut self, expression: impl AsRef<str>) -> Self {
        self.condition_expression = Some(Arc::<str>::from(expression.as_ref()));
        self
    }
}
