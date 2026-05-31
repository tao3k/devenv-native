//! Workflow-neutral Qianji control-plane contracts.
//!
//! This crate owns run and step management state, not workflow semantics.
//! Workflow engines, BPMN adapters, and Agent workers should depend on this
//! crate when they need auditable lifecycle, lease, evidence, gate, and cost
//! tracking.

mod activity_journal;
mod activity_queue;
mod activity_schedule_plan;
mod admission;
mod agent;
mod agent_journal;
mod approval;
mod cost_inventory;
#[cfg(feature = "duckdb")]
mod duckdb_ledger;
mod error;
mod event;
mod gate;
mod heartbeat_journal;
mod identity;
mod journal_batch;
mod lease_journal;
mod llm_inventory;
mod memory;
mod model;
mod observation_journal;
mod operator_summary;
mod policy;
mod recovery;
mod recovery_applier;
mod recovery_journal;
mod recovery_loop;
mod recovery_plan;
mod recovery_snapshot;
mod run_journal;
mod signal_inventory;
mod signal_journal;
mod step_lifecycle_journal;
mod step_queue_journal;
mod timer_inventory;
mod timer_journal;
mod tool;
mod traits;
#[cfg(feature = "valkey")]
mod valkey_hot_state;
mod view;
mod worker_lifecycle;
mod workflow_decision;
mod workflow_observation;
mod workflow_trace_journal;

#[cfg(feature = "duckdb")]
pub use duckdb_ledger::DuckDbControlLedger;
#[cfg(feature = "valkey")]
pub use valkey_hot_state::{ValkeyHotStateConfig, ValkeyHotStateStore, ValkeyKeyNamespace};
pub use {
    activity_journal::{
        ActivityCompletedJournalRecord, ActivityFailedJournalRecord, ActivityJournalScope,
        ActivityJournalWriteOutcome, ActivityJournalWriteStatus, ActivityStartedJournalRecord,
        AdmittedActivityScheduleRecord, AdmittedActivityTaskScheduleRecord,
        AdmittedLlmActivityScheduleRecord, record_activity_completed,
        record_activity_completed_idempotent, record_activity_failed,
        record_activity_failed_idempotent, record_activity_started,
        record_activity_started_idempotent, record_admitted_activity_schedule,
        record_admitted_activity_schedule_idempotent, record_admitted_activity_task_schedule,
        record_admitted_activity_task_schedule_idempotent, record_admitted_llm_activity_schedule,
        record_admitted_llm_activity_schedule_idempotent,
    },
    activity_queue::{
        ActivityQueueItem, ActivityQueueProjection, ActivityQueueSummary,
        WorkerActivityHotStateMirrorOutcome, WorkerActivityHotStateMirrorRequest,
        WorkerActivityTask, mirror_worker_activity_tasks_to_hot_state,
    },
    activity_schedule_plan::{
        ACTIVITY_SCHEDULE_ADMISSION_KIND, ACTIVITY_SCHEDULE_ADMISSION_PENDING_STATUS,
        ACTIVITY_SCHEDULE_ADMISSION_PLAN_CONTRACT, ActivityScheduleAdmissionExecutionFlags,
        ActivityScheduleAdmissionInputExecutionFlags, ActivityScheduleAdmissionKind,
        ActivityScheduleAdmissionPlanItem, ActivityScheduleAdmissionRuntimeExecutionFlags,
        ActivityScheduleAdmissionSafetyFlags, ActivityScheduleAdmissionStatus,
        ActivitySchedulePlanAdmissionItemOutcome, ActivitySchedulePlanAdmissionReport,
        ActivitySchedulePlanAdmissionRequest, admit_activity_schedule_plan,
        parse_activity_schedule_plan_json,
    },
    admission::{LlmActivityAdmission, ToolActivityAdmission},
    agent::{AgentDecision, AgentDecisionOutcome, AgentProposal},
    agent_journal::{
        AgentDecisionJournalRecord, AgentJournalScope, AgentProposalJournalRecord,
        record_agent_decision, record_agent_proposal,
    },
    approval::{
        HumanApprovalDecision, HumanApprovalDecisionStatus, HumanApprovalRequest,
        HumanApprovalResolution,
    },
    cost_inventory::{CostInventoryItem, CostInventoryProjection, CostInventorySummary},
    error::{ControlError, ControlResult},
    event::{ControlEvent, ControlEventKind, ControlEventRecord, RecoveryAttempt},
    gate::RequiredEvidenceGate,
    heartbeat_journal::{
        WorkerHeartbeatJournalRecord, record_worker_heartbeat,
        record_worker_heartbeat_with_hot_state,
    },
    identity::{
        ActivityId, ActivityType, AgentDecisionId, AgentProposalId, ApprovalRequestId, ApproverId,
        ArtifactId, ArtifactKind, DecisionReasonCode, ErrorCode, EvidenceId, GateName,
        IdempotencyKey, LeaseId, LlmModelId, PermissionScope, RunId, SignalName, StepId, TaskQueue,
        TimerId, TokenId, ToolName, VersionKey, WorkerId,
    },
    journal_batch::{ControlJournalBatchRecordingOutcome, record_control_event_batch},
    lease_journal::{StepLeaseReleaseJournalRecord, record_step_lease_released},
    llm_inventory::{
        LlmActivityInventoryItem, LlmActivityInventoryProjection, LlmActivityInventorySummary,
    },
    memory::{InMemoryControlLedger, InMemoryHotStateStore},
    model::{
        ActivityFailure, ActivityResult, ActivityRetryDecision, ActivityRetryPolicy,
        ActivityRetryStopReason, ActivityTask, ActivityTaskLease, ArtifactRef, Budget,
        CostObservation, EvidenceRef, GateResult, HotStateLeasedActivityTask, HotStateLeasedStep,
        HotStateSnapshot, LlmActivityRequest, LlmActivityTask, RecoveryPolicy,
        RunScopedActivityTaskClaimRequest, RunStatus, RunnableActivityTask, RunnableStep,
        SignalRecord, StepLease, StepStatus, TimerRecord, VersionPin, WaitReason, WorkerHeartbeat,
        WorkerRef,
    },
    observation_journal::{
        CostObservationJournalRecord, StepEvidenceJournalRecord, StepGateResultJournalRecord,
        record_cost_observation, record_step_evidence, record_step_gate_result,
    },
    operator_summary::{RunOperatorDiagnostics, RunOperatorSummary},
    policy::{AgentPolicyReason, ToolPolicyReduction, ToolPolicyReductionRequest},
    recovery::{
        ActivityRecoveryItem, AgentDecisionRecoveryItem, FailedActivityRecoveryItem,
        LeaseRecoveryItem, RecoveryItemScope, RunRecoveryView, StepRecoveryItem, TimerRecoveryItem,
    },
    recovery_applier::{
        RecoveryActionApplication, RecoveryActionApplicationReason,
        RecoveryActionApplicationRequest, apply_recovery_action,
    },
    recovery_journal::{RecoveryStartedJournalRecord, record_recovery_started},
    recovery_loop::{
        RecoveryLoopActionApplication, RecoveryLoopApplication, RecoveryLoopApplicationRequest,
        apply_recovery_plan,
    },
    recovery_plan::{RecoveryPlanAction, RunRecoveryPlan, RunRecoveryPlanSummary},
    recovery_snapshot::RunRecoverySnapshot,
    run_journal::{
        RunAdmittedJournalRecord, RunCreatedJournalRecord, RunPlanRecordedJournalRecord,
        RunTerminalJournalRecord, RunTerminalJournalStatus, record_run_admitted,
        record_run_created, record_run_plan_recorded, record_run_terminal,
    },
    signal_inventory::{SignalInventoryItem, SignalInventoryProjection, SignalInventorySummary},
    signal_journal::{SignalReceiveJournalRecord, record_signal_received},
    step_lifecycle_journal::{
        StepCreatedJournalRecord, StepFailureJournalInput, StepStartedJournalRecord,
        StepTerminalJournalRecord, StepTerminalJournalStatus, StepToolCallJournalRecord,
        record_step_created, record_step_started, record_step_terminal, record_step_tool_call,
    },
    step_queue_journal::{
        StepQueueJournalRecord, record_step_queued, record_step_queued_with_hot_state,
    },
    timer_inventory::{TimerInventoryItem, TimerInventoryProjection, TimerInventorySummary},
    timer_journal::{TimerFireJournalRecord, record_timer_fired},
    tool::{
        ToolActivityContract, ToolAuthorizationDecision, ToolPermissionDecision,
        ToolPermissionMode, ToolRiskLevel,
    },
    traits::{ControlLedger, EvidenceGate, HotStateStore},
    view::{
        ActivityStatus, ActivityView, RunView, StepView, TimerStatus, TimerView, replay_run_view,
    },
    worker_lifecycle::{
        WorkerActivityCompletedRecord, WorkerActivityFailedRecord, WorkerActivityFailureInput,
        WorkerActivityStartRecord, record_worker_activity_completed_idempotent,
        record_worker_activity_failed_idempotent, record_worker_activity_started_idempotent,
    },
    workflow_decision::{
        WorkflowStageDecisionRecord, WorkflowStageDecisionRecordingOutcome,
        WorkflowStageDecisionRecordingRequest, WorkflowStageRecoveryDecisionRecord,
        WorkflowStageRecoveryDecisionRecordingRequest, record_workflow_stage_decision,
        record_workflow_stage_recovery_decision,
    },
    workflow_observation::{
        WorkflowControlEvidenceRequirements, WorkflowRunCostObservationRecordingRequest,
        WorkflowRunRecoveryAttemptRecordingRequest, WorkflowStageCostObservationRecordingRequest,
        WorkflowStageEvidenceRecordingRequest, WorkflowStageGateResultRecordingRequest,
        WorkflowStageRecoveryAttemptRecordingRequest, record_workflow_run_cost_observation,
        record_workflow_run_recovery_attempt, record_workflow_stage_cost_observation,
        record_workflow_stage_evidence, record_workflow_stage_gate_result,
        record_workflow_stage_recovery_attempt,
    },
    workflow_trace_journal::{
        WorkflowTraceProjectionRecord, WorkflowTraceProjectionStage,
        WorkflowTraceProjectionStageInput, WorkflowTraceProjectionStageStatus,
        record_workflow_trace_projection,
    },
};
