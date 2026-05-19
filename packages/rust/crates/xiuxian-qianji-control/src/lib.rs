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
mod admission;
mod agent;
mod agent_journal;
mod approval;
#[cfg(feature = "duckdb")]
mod duckdb_ledger;
mod error;
mod event;
mod gate;
mod heartbeat_journal;
mod identity;
mod memory;
mod model;
mod policy;
mod recovery;
mod recovery_plan;
mod recovery_snapshot;
mod tool;
mod traits;
#[cfg(feature = "valkey")]
mod valkey_hot_state;
mod view;

pub use activity_journal::{
    ActivityCompletedJournalRecord, ActivityFailedJournalRecord, ActivityJournalScope,
    ActivityJournalWriteOutcome, ActivityJournalWriteStatus, ActivityStartedJournalRecord,
    record_activity_completed, record_activity_completed_idempotent, record_activity_failed,
    record_activity_failed_idempotent, record_activity_started, record_activity_started_idempotent,
};
pub use activity_journal::{
    AdmittedActivityScheduleRecord, record_admitted_activity_schedule,
    record_admitted_activity_schedule_idempotent,
};
pub use admission::ToolActivityAdmission;
pub use agent::{AgentDecision, AgentDecisionOutcome, AgentProposal};
pub use agent_journal::{
    AgentDecisionJournalRecord, AgentJournalScope, AgentProposalJournalRecord,
    record_agent_decision, record_agent_proposal,
};
pub use approval::{
    HumanApprovalDecision, HumanApprovalDecisionStatus, HumanApprovalRequest,
    HumanApprovalResolution,
};
#[cfg(feature = "duckdb")]
pub use duckdb_ledger::DuckDbControlLedger;
pub use error::{ControlError, ControlResult};
pub use event::{ControlEvent, ControlEventKind, ControlEventRecord, RecoveryAttempt};
pub use gate::RequiredEvidenceGate;
pub use heartbeat_journal::{
    WorkerHeartbeatJournalRecord, record_worker_heartbeat, record_worker_heartbeat_with_hot_state,
};
pub use identity::{
    ActivityId, ActivityType, AgentDecisionId, AgentProposalId, ApprovalRequestId, ApproverId,
    ArtifactId, ArtifactKind, DecisionReasonCode, ErrorCode, EvidenceId, GateName, IdempotencyKey,
    LeaseId, LlmModelId, PermissionScope, RunId, SignalName, StepId, TaskQueue, TimerId, TokenId,
    ToolName, VersionKey, WorkerId,
};
pub use memory::{InMemoryControlLedger, InMemoryHotStateStore};
pub use model::{
    ActivityFailure, ActivityResult, ActivityRetryDecision, ActivityRetryPolicy,
    ActivityRetryStopReason, ActivityTask, ArtifactRef, Budget, CostObservation, EvidenceRef,
    GateResult, LlmActivityRequest, LlmActivityTask, RecoveryPolicy, RunStatus, RunnableStep,
    SignalRecord, StepLease, StepStatus, TimerRecord, VersionPin, WaitReason, WorkerHeartbeat,
    WorkerRef,
};
pub use policy::{AgentPolicyReason, ToolPolicyReduction, ToolPolicyReductionRequest};
pub use recovery::{
    ActivityRecoveryItem, AgentDecisionRecoveryItem, FailedActivityRecoveryItem, LeaseRecoveryItem,
    RecoveryItemScope, RunRecoveryView, StepRecoveryItem, TimerRecoveryItem,
};
pub use recovery_plan::{RecoveryPlanAction, RunRecoveryPlan, RunRecoveryPlanSummary};
pub use recovery_snapshot::RunRecoverySnapshot;
pub use tool::{
    ToolActivityContract, ToolAuthorizationDecision, ToolPermissionDecision, ToolPermissionMode,
    ToolRiskLevel,
};
pub use traits::{ControlLedger, EvidenceGate, HotStateStore};
#[cfg(feature = "valkey")]
pub use valkey_hot_state::{ValkeyHotStateConfig, ValkeyHotStateStore, ValkeyKeyNamespace};
pub use view::{
    ActivityStatus, ActivityView, RunView, StepView, TimerStatus, TimerView, replay_run_view,
};
