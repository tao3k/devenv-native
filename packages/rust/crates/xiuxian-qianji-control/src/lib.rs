//! Workflow-neutral Qianji control-plane contracts.
//!
//! This crate owns run and step management state, not workflow semantics.
//! Workflow engines, BPMN adapters, and Agent workers should depend on this
//! crate when they need auditable lifecycle, lease, evidence, gate, and cost
//! tracking.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public control-plane API"),
        )
    }
);

mod activity_journal;
mod activity_queue;
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
mod lease_journal;
mod llm_inventory;
mod memory;
mod model;
mod operator_summary;
mod policy;
mod recovery;
mod recovery_applier;
mod recovery_journal;
mod recovery_loop;
mod recovery_plan;
mod recovery_snapshot;
mod signal_inventory;
mod step_queue_journal;
mod timer_inventory;
mod timer_journal;
mod tool;
mod traits;
#[cfg(feature = "valkey")]
mod valkey_hot_state;
mod view;
mod worker_lifecycle;

#[cfg(feature = "duckdb")]
pub use duckdb_ledger::DuckDbControlLedger;
#[cfg(feature = "valkey")]
pub use valkey_hot_state::{ValkeyHotStateConfig, ValkeyHotStateStore, ValkeyKeyNamespace};
pub use {
    activity_journal::{
        ActivityCompletedJournalRecord, ActivityFailedJournalRecord, ActivityJournalScope,
        ActivityJournalWriteOutcome, ActivityJournalWriteStatus, ActivityStartedJournalRecord,
        AdmittedActivityScheduleRecord, AdmittedLlmActivityScheduleRecord,
        record_activity_completed, record_activity_completed_idempotent, record_activity_failed,
        record_activity_failed_idempotent, record_activity_started,
        record_activity_started_idempotent, record_admitted_activity_schedule,
        record_admitted_activity_schedule_idempotent, record_admitted_llm_activity_schedule,
        record_admitted_llm_activity_schedule_idempotent,
    },
    activity_queue::{
        ActivityQueueItem, ActivityQueueProjection, ActivityQueueSummary,
        WorkerActivityHotStateMirrorOutcome, WorkerActivityHotStateMirrorRequest,
        WorkerActivityTask, mirror_worker_activity_tasks_to_hot_state,
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
    lease_journal::{StepLeaseReleaseJournalRecord, record_step_lease_released},
    llm_inventory::{
        LlmActivityInventoryItem, LlmActivityInventoryProjection, LlmActivityInventorySummary,
    },
    memory::{InMemoryControlLedger, InMemoryHotStateStore},
    model::{
        ActivityFailure, ActivityResult, ActivityRetryDecision, ActivityRetryPolicy,
        ActivityRetryStopReason, ActivityTask, ActivityTaskLease, ArtifactRef, Budget,
        CostObservation, EvidenceRef, GateResult, HotStateLeasedActivityTask, HotStateLeasedStep,
        HotStateSnapshot, LlmActivityRequest, LlmActivityTask, RecoveryPolicy, RunStatus,
        RunnableActivityTask, RunnableStep, SignalRecord, StepLease, StepStatus, TimerRecord,
        VersionPin, WaitReason, WorkerHeartbeat, WorkerRef,
    },
    operator_summary::RunOperatorSummary,
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
    signal_inventory::{SignalInventoryItem, SignalInventoryProjection, SignalInventorySummary},
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
};
