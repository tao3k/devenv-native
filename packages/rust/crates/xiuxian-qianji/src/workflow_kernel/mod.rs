//! Low-overhead Rust-native workflow kernel.

mod checkpoint;
mod model;
mod run;
mod stage;
mod topology;

#[cfg(test)]
mod tests;

pub use checkpoint::{
    WorkflowCheckpointError, WorkflowCheckpointRef, WorkflowCheckpointStorageKind,
    WorkflowMemoryCheckpointStore,
};
pub use model::{
    WorkflowEdgeKind, WorkflowExecutionReport, WorkflowStageFacts, WorkflowStageStatus,
    WorkflowStageTrace, WorkflowTrace,
};
pub use run::{WorkflowExecutionError, WorkflowRun};
pub use stage::WorkflowStage;
pub use topology::{
    WorkflowCompletionError, WorkflowStageBinding, WorkflowTopology, WorkflowTopologyEdge,
    WorkflowTopologyError,
};
