//! Low-overhead Rust-native workflow kernel.

mod checkpoint;
mod control;
mod model;
mod run;
mod stage;
mod topology;

#[cfg(test)]
#[path = "../../tests/unit/workflow_kernel/mod.rs"]
mod tests;

pub use checkpoint::{
    WorkflowCheckpointError, WorkflowCheckpointRef, WorkflowCheckpointStorageKind,
    WorkflowMemoryCheckpointStore, WorkflowStageCheckpointMiss,
};
pub use control::{workflow_trace_to_control_event_records, workflow_trace_to_control_events};
pub use model::{
    WorkflowCheckpointId, WorkflowEdgeKind, WorkflowExecutionReport, WorkflowId,
    WorkflowStageFacts, WorkflowStageId, WorkflowStageStatus, WorkflowStageTrace, WorkflowTrace,
};
pub use run::{
    WorkflowBoundedFanoutStageRequest, WorkflowExecutionError, WorkflowMemoryCheckpointRecord,
    WorkflowRun,
};
pub use stage::WorkflowStage;
pub use topology::{
    WorkflowCompletionError, WorkflowDuplicateStage, WorkflowMissingEdgeStage,
    WorkflowStageBinding, WorkflowTopology, WorkflowTopologyEdge, WorkflowTopologyError,
};
