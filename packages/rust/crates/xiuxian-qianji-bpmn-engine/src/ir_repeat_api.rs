//! Public BPMN repeat contract owner.

use std::sync::Arc;

/// Immutable repeatable-task snapshot for one BPMN node.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BpmnRepeatSpec {
    /// Standard loop characteristics attached to one host-blocking task.
    StandardLoop(BpmnStandardLoopSpec),
    /// Sequential multi-instance characteristics with bounded cardinality.
    SequentialMultiInstance(BpmnSequentialMultiInstanceSpec),
    /// Parallel multi-instance characteristics with bounded cardinality.
    ParallelMultiInstance(BpmnParallelMultiInstanceSpec),
}

/// Immutable multi-instance data-binding snapshot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnMultiInstanceDataBindingSpec {
    /// Variable path that resolves to the source array or object collection.
    pub loop_data_input_ref: Arc<str>,
    /// Per-iteration variable name bound to the current source item.
    pub input_data_item: Arc<str>,
    /// Optional destination variable path for aggregated multi-instance output.
    pub loop_data_output_ref: Option<Arc<str>>,
    /// Optional per-iteration output item name to collect into the destination.
    pub output_data_item: Option<Arc<str>>,
}

impl BpmnMultiInstanceDataBindingSpec {
    /// Creates a collection-backed multi-instance data-binding snapshot.
    #[must_use]
    pub fn new(loop_data_input_ref: impl AsRef<str>, input_data_item: impl AsRef<str>) -> Self {
        Self {
            loop_data_input_ref: Arc::<str>::from(loop_data_input_ref.as_ref()),
            input_data_item: Arc::<str>::from(input_data_item.as_ref()),
            loop_data_output_ref: None,
            output_data_item: None,
        }
    }

    /// Attaches one bounded output aggregation target.
    #[must_use]
    pub fn with_output(
        mut self,
        loop_data_output_ref: impl AsRef<str>,
        output_data_item: impl AsRef<str>,
    ) -> Self {
        self.loop_data_output_ref = Some(Arc::<str>::from(loop_data_output_ref.as_ref()));
        self.output_data_item = Some(Arc::<str>::from(output_data_item.as_ref()));
        self
    }
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
    /// Optional fixed sequential iteration count.
    pub loop_cardinality: Option<u32>,
    /// Optional collection-backed data-binding expansion.
    pub data_binding: Option<BpmnMultiInstanceDataBindingSpec>,
    /// Optional bounded completion condition snapshot.
    pub completion_condition: Option<Arc<str>>,
}

impl BpmnSequentialMultiInstanceSpec {
    /// Creates a sequential multi-instance snapshot.
    #[must_use]
    pub fn new(loop_cardinality: u32) -> Self {
        Self {
            loop_cardinality: Some(loop_cardinality),
            data_binding: None,
            completion_condition: None,
        }
    }

    /// Creates a collection-backed sequential multi-instance snapshot.
    #[must_use]
    pub fn from_data_binding(data_binding: BpmnMultiInstanceDataBindingSpec) -> Self {
        Self {
            loop_cardinality: None,
            data_binding: Some(data_binding),
            completion_condition: None,
        }
    }

    /// Attaches an optional bounded completion-condition snapshot.
    #[must_use]
    pub fn with_completion_condition(mut self, completion_condition: impl AsRef<str>) -> Self {
        self.completion_condition = Some(Arc::<str>::from(completion_condition.as_ref()));
        self
    }
}

/// Immutable parallel multi-instance snapshot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnParallelMultiInstanceSpec {
    /// Optional fixed parallel iteration count.
    pub loop_cardinality: Option<u32>,
    /// Optional collection-backed data-binding expansion.
    pub data_binding: Option<BpmnMultiInstanceDataBindingSpec>,
    /// Optional bounded completion condition snapshot.
    pub completion_condition: Option<Arc<str>>,
}

impl BpmnParallelMultiInstanceSpec {
    /// Creates a parallel multi-instance snapshot.
    #[must_use]
    pub fn new(loop_cardinality: u32) -> Self {
        Self {
            loop_cardinality: Some(loop_cardinality),
            data_binding: None,
            completion_condition: None,
        }
    }

    /// Creates a collection-backed parallel multi-instance snapshot.
    #[must_use]
    pub fn from_data_binding(data_binding: BpmnMultiInstanceDataBindingSpec) -> Self {
        Self {
            loop_cardinality: None,
            data_binding: Some(data_binding),
            completion_condition: None,
        }
    }

    /// Attaches an optional bounded completion-condition snapshot.
    #[must_use]
    pub fn with_completion_condition(mut self, completion_condition: impl AsRef<str>) -> Self {
        self.completion_condition = Some(Arc::<str>::from(completion_condition.as_ref()));
        self
    }
}
