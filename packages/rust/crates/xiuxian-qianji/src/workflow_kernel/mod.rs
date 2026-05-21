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
pub use control::{
    WorkflowControlEvidenceRequirements, WorkflowControlRecorder, WorkflowControlRecordingOutcome,
    WorkflowControlRecordingPolicy, WorkflowStageDecisionRecord,
    WorkflowStageDecisionRecordingOutcome, WorkflowStageDecisionRecordingRequest,
    WorkflowStageRecoveryDecisionRecord, WorkflowStageRecoveryDecisionRecordingRequest,
    record_workflow_run_cost_observation, record_workflow_run_recovery_attempt,
    record_workflow_stage_cost_observation, record_workflow_stage_decision,
    record_workflow_stage_evidence, record_workflow_stage_gate_result,
    record_workflow_stage_recovery_attempt, record_workflow_stage_recovery_decision,
    record_workflow_trace_to_control_ledger,
    record_workflow_trace_to_control_ledger_with_required_evidence,
    workflow_trace_to_control_event_records,
    workflow_trace_to_control_event_records_with_required_evidence,
    workflow_trace_to_control_events, workflow_trace_to_control_events_with_required_evidence,
};
pub use control::{
    WorkflowRunCostObservationRecordingRequest, WorkflowRunRecoveryAttemptRecordingRequest,
    WorkflowStageCostObservationRecordingRequest, WorkflowStageEvidenceRecordingRequest,
    WorkflowStageGateResultRecordingRequest, WorkflowStageRecoveryAttemptRecordingRequest,
};
pub use model::{
    WorkflowCheckpointId, WorkflowEdgeKind, WorkflowExecutionReport, WorkflowId,
    WorkflowStageFacts, WorkflowStageId, WorkflowStageStatus, WorkflowStageTrace, WorkflowTrace,
};
pub use run::{
    WorkflowBoundedFanoutStageRequest, WorkflowCheckedControlRecordingError,
    WorkflowCheckedControlRecordingFailure, WorkflowControlRecordedReport,
    WorkflowControlRecordingFailure, WorkflowExecutionError, WorkflowMemoryCheckpointRecord,
    WorkflowRun,
};
pub use stage::WorkflowStage;
pub use topology::{
    WorkflowCompletionError, WorkflowDuplicateStage, WorkflowMissingEdgeStage,
    WorkflowStageBinding, WorkflowTopology, WorkflowTopologyEdge, WorkflowTopologyError,
};
