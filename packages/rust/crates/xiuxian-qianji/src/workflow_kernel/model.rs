//! Workflow kernel data model.

use super::{WorkflowCheckpointRef, WorkflowMemoryCheckpointStore};

/// Describes the payload kind carried across one workflow stage boundary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkflowEdgeKind {
    /// A normal Rust type carried inside the current process.
    Typed {
        /// Human-readable type or contract name.
        type_name: String,
    },
    /// An Arrow-backed edge that can be handed to Flight, FFI, or another
    /// Arrow-compatible boundary without changing the logical schema.
    ArrowRecordBatch {
        /// Stable logical schema name.
        schema_name: String,
        /// Stable logical schema version.
        schema_version: String,
    },
}

impl WorkflowEdgeKind {
    /// Builds a typed Rust edge descriptor.
    #[must_use]
    pub fn typed(type_name: impl Into<String>) -> Self {
        Self::Typed {
            type_name: type_name.into(),
        }
    }

    /// Builds an Arrow `RecordBatch` edge descriptor.
    #[must_use]
    pub fn arrow_record_batch(
        schema_name: impl Into<String>,
        schema_version: impl Into<String>,
    ) -> Self {
        Self::ArrowRecordBatch {
            schema_name: schema_name.into(),
            schema_version: schema_version.into(),
        }
    }
}

/// Optional metrics attached to a workflow stage input or output edge.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowStageFacts {
    /// Optional payload kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_kind: Option<WorkflowEdgeKind>,
    /// Optional item or row count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_count: Option<usize>,
    /// Optional cache hit count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_hit_count: Option<usize>,
}

impl WorkflowStageFacts {
    /// Records a typed payload kind.
    #[must_use]
    pub fn typed(type_name: impl Into<String>) -> Self {
        Self {
            edge_kind: Some(WorkflowEdgeKind::typed(type_name)),
            ..Self::default()
        }
    }

    /// Records an Arrow-backed payload kind.
    #[must_use]
    pub fn arrow_record_batch(
        schema_name: impl Into<String>,
        schema_version: impl Into<String>,
    ) -> Self {
        Self {
            edge_kind: Some(WorkflowEdgeKind::arrow_record_batch(
                schema_name,
                schema_version,
            )),
            ..Self::default()
        }
    }

    /// Records an item or row count.
    #[must_use]
    pub fn with_item_count(mut self, item_count: usize) -> Self {
        self.item_count = Some(item_count);
        self
    }

    /// Records a cache hit count.
    #[must_use]
    pub fn with_cache_hit_count(mut self, cache_hit_count: usize) -> Self {
        self.cache_hit_count = Some(cache_hit_count);
        self
    }
}

/// Terminal status for one stage execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStageStatus {
    /// Stage completed successfully.
    Succeeded,
    /// Stage returned an error.
    Failed,
}

/// Trace row for one workflow stage execution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowStageTrace {
    /// Stable stage identifier.
    pub stage_id: String,
    /// Stage terminal status.
    pub status: WorkflowStageStatus,
    /// Unix timestamp in milliseconds when the stage began.
    pub started_unix_ms: u64,
    /// Stage wall-clock duration in nanoseconds.
    pub duration_nanos: u64,
    /// Input edge facts captured before execution.
    pub input: WorkflowStageFacts,
    /// Output edge facts captured after successful execution.
    pub output: WorkflowStageFacts,
    /// Stage error message when status is `failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Same-process checkpoint references produced by this stage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checkpoints: Vec<WorkflowCheckpointRef>,
}

/// Trace for one workflow execution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowTrace {
    /// Stable workflow identifier.
    pub workflow_id: String,
    /// Ordered stage traces.
    pub stages: Vec<WorkflowStageTrace>,
}

impl WorkflowTrace {
    /// Creates an empty workflow trace.
    #[must_use]
    pub fn new(workflow_id: impl Into<String>) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            stages: Vec::new(),
        }
    }
}

/// Final output and trace from one workflow execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowExecutionReport<T> {
    /// Final typed workflow output.
    pub output: T,
    /// Ordered workflow execution trace.
    pub trace: WorkflowTrace,
    /// Same-process memory checkpoints retained by the run.
    pub memory_checkpoints: WorkflowMemoryCheckpointStore,
}
