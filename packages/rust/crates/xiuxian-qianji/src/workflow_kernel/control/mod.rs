//! Control-plane event mapping for workflow-kernel traces.

mod decision;
mod projection;
mod recording;

pub use decision::{
    WorkflowStageDecisionRecord, WorkflowStageDecisionRecordingOutcome,
    WorkflowStageDecisionRecordingRequest, WorkflowStageRecoveryDecisionRecord,
    WorkflowStageRecoveryDecisionRecordingRequest, record_workflow_stage_decision,
    record_workflow_stage_recovery_decision,
};
pub use projection::{
    WorkflowRunCostObservationRecordingRequest, WorkflowRunRecoveryAttemptRecordingRequest,
    WorkflowStageCostObservationRecordingRequest, WorkflowStageEvidenceRecordingRequest,
    WorkflowStageGateResultRecordingRequest, WorkflowStageRecoveryAttemptRecordingRequest,
    record_workflow_run_cost_observation, record_workflow_run_recovery_attempt,
    record_workflow_stage_cost_observation, record_workflow_stage_evidence,
    record_workflow_stage_gate_result, record_workflow_stage_recovery_attempt,
};
pub use recording::{
    WorkflowControlEvidenceRequirements, WorkflowControlRecorder, WorkflowControlRecordingOutcome,
    WorkflowControlRecordingPolicy, record_workflow_trace_to_control_ledger,
    record_workflow_trace_to_control_ledger_with_required_evidence,
    workflow_trace_to_control_event_records,
    workflow_trace_to_control_event_records_with_required_evidence,
    workflow_trace_to_control_events, workflow_trace_to_control_events_with_required_evidence,
};
