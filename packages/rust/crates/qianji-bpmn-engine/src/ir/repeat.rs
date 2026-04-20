//! BPMN repeatable-task snapshot types.

use std::sync::Arc;

/// Immutable repeatable-task snapshot for one BPMN node.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BpmnRepeatSpec {
    /// Standard loop characteristics attached to one host-blocking task.
    StandardLoop(BpmnStandardLoopSpec),
    /// Sequential multi-instance characteristics with bounded cardinality.
    SequentialMultiInstance(BpmnSequentialMultiInstanceSpec),
}

/// Immutable standard-loop snapshot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnStandardLoopSpec {
    /// Whether the continuation condition is checked before each iteration.
    pub test_before: bool,
    /// Optional maximum iteration count.
    pub loop_maximum: Option<u32>,
    /// Optional source-level loop condition snapshot.
    pub loop_condition: Option<Arc<str>>,
}

impl BpmnStandardLoopSpec {
    /// Creates a standard-loop snapshot.
    #[must_use]
    pub fn new(test_before: bool, loop_maximum: Option<u32>) -> Self {
        Self {
            test_before,
            loop_maximum,
            loop_condition: None,
        }
    }

    /// Attaches an optional source-level loop condition snapshot.
    #[must_use]
    pub fn with_loop_condition(mut self, loop_condition: impl AsRef<str>) -> Self {
        self.loop_condition = Some(Arc::<str>::from(loop_condition.as_ref()));
        self
    }
}

/// Immutable sequential multi-instance snapshot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnSequentialMultiInstanceSpec {
    /// Total number of sequential iterations to execute.
    pub loop_cardinality: u32,
}

impl BpmnSequentialMultiInstanceSpec {
    /// Creates a sequential multi-instance snapshot.
    #[must_use]
    pub fn new(loop_cardinality: u32) -> Self {
        Self { loop_cardinality }
    }
}
