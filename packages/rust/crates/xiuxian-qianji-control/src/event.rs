//! Append-only control event model.

use crate::{
    ActivityFailure, ActivityId, ActivityResult, ActivityTask, AgentDecision, AgentProposal,
    ArtifactRef, Budget, CostObservation, EvidenceRef, GateResult, RecoveryPolicy, RunId,
    SignalRecord, StepId, StepLease, TimerId, TimerRecord, VersionPin, WaitReason, WorkerHeartbeat,
};

/// Stored control event with a ledger-assigned sequence.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ControlEventRecord {
    /// Monotonic ledger sequence.
    pub sequence: u64,
    /// Stored event payload.
    pub event: ControlEvent,
}

/// One append-only control event.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ControlEvent {
    /// Owning run id.
    pub run_id: RunId,
    /// Optional step id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<StepId>,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Event kind.
    pub kind: ControlEventKind,
}

impl ControlEvent {
    /// Creates a run-scoped event.
    #[must_use]
    pub fn run(run_id: RunId, occurred_at_ms: u64, kind: ControlEventKind) -> Self {
        Self {
            run_id,
            step_id: None,
            occurred_at_ms,
            kind,
        }
    }

    /// Creates a step-scoped event.
    #[must_use]
    pub fn step(
        run_id: RunId,
        step_id: StepId,
        occurred_at_ms: u64,
        kind: ControlEventKind,
    ) -> Self {
        Self {
            run_id,
            step_id: Some(step_id),
            occurred_at_ms,
            kind,
        }
    }
}

/// Specific control event payload.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ControlEventKind {
    /// A run intent was created.
    RunCreated {
        /// User or system intent.
        intent: String,
        /// Optional run budget.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        budget: Option<Budget>,
        /// Extension metadata.
        #[serde(default)]
        metadata: serde_json::Value,
    },
    /// A run was admitted by the planner.
    RunAdmitted,
    /// A plan summary was recorded.
    PlanRecorded {
        /// Human-readable plan summary.
        summary: String,
    },
    /// A step was declared.
    StepCreated {
        /// Human-readable step label.
        title: String,
        /// Required evidence keys.
        #[serde(default)]
        required_evidence: Vec<String>,
        /// Optional step budget.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        budget: Option<Budget>,
    },
    /// A step was queued in hot state.
    StepQueued,
    /// A step lease was acquired.
    StepLeaseAcquired {
        /// Active lease.
        lease: StepLease,
    },
    /// A step lease was renewed.
    StepLeaseRenewed {
        /// Renewed lease.
        lease: StepLease,
    },
    /// A step lease was released.
    StepLeaseReleased {
        /// Released lease.
        lease: StepLease,
    },
    /// A step started execution.
    StepStarted,
    /// A step entered a wait state.
    StepWaiting {
        /// Wait reason.
        reason: WaitReason,
    },
    /// A tool call was recorded.
    ToolCallRecorded {
        /// Tool name.
        tool_name: String,
        /// Extension metadata.
        #[serde(default)]
        metadata: serde_json::Value,
    },
    /// An Agent proposal was recorded.
    AgentProposalRecorded {
        /// Agent proposal payload.
        proposal: AgentProposal,
    },
    /// A deterministic Agent decision was recorded.
    AgentDecisionRecorded {
        /// Agent decision payload.
        decision: AgentDecision,
    },
    /// An external side-effect activity was scheduled.
    ActivityScheduled {
        /// Scheduled activity task.
        task: ActivityTask,
    },
    /// An activity worker started an attempt.
    ActivityStarted {
        /// Activity id.
        activity_id: ActivityId,
        /// Worker that started the attempt, when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        worker_id: Option<crate::WorkerId>,
        /// Attempt number, starting at one.
        attempt: u32,
    },
    /// An activity completed successfully.
    ActivityCompleted {
        /// Activity id.
        activity_id: ActivityId,
        /// Completion result.
        result: ActivityResult,
    },
    /// An activity failed.
    ActivityFailed {
        /// Activity id.
        activity_id: ActivityId,
        /// Failure payload.
        failure: ActivityFailure,
    },
    /// An external signal was received.
    SignalReceived {
        /// Signal payload and metadata.
        signal: SignalRecord,
    },
    /// A durable timer was scheduled.
    TimerScheduled {
        /// Scheduled timer.
        timer: TimerRecord,
    },
    /// A durable timer fired.
    TimerFired {
        /// Timer id.
        timer_id: TimerId,
    },
    /// A deterministic version or schema fact was pinned.
    VersionPinned {
        /// Version pin.
        pin: VersionPin,
    },
    /// An artifact was attached.
    ArtifactAttached {
        /// Artifact reference.
        artifact: ArtifactRef,
    },
    /// Evidence was attached.
    EvidenceAttached {
        /// Evidence reference.
        evidence: EvidenceRef,
    },
    /// Cost was observed.
    CostObserved {
        /// Cost observation.
        observation: CostObservation,
    },
    /// A gate was evaluated.
    GateEvaluated {
        /// Gate result.
        result: GateResult,
    },
    /// Recovery started.
    RecoveryStarted {
        /// Recovery attempt.
        attempt: RecoveryAttempt,
    },
    /// A heartbeat was observed.
    WorkerHeartbeatObserved {
        /// Worker heartbeat.
        heartbeat: WorkerHeartbeat,
    },
    /// A step succeeded.
    StepSucceeded,
    /// A step failed.
    StepFailed {
        /// Error code.
        error_code: String,
        /// Error message.
        message: String,
        /// Whether retry is allowed.
        retryable: bool,
    },
    /// A step was blocked.
    StepBlocked {
        /// Block reason.
        reason: String,
    },
    /// A step was cancelled.
    StepCancelled {
        /// Cancel reason.
        reason: String,
    },
    /// A run completed.
    RunCompleted,
    /// A run failed.
    RunFailed {
        /// Error message.
        message: String,
    },
    /// A run was blocked.
    RunBlocked {
        /// Block reason.
        reason: String,
    },
    /// A run was aborted.
    RunAborted {
        /// Abort reason.
        reason: String,
    },
}

/// Recovery attempt record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecoveryAttempt {
    /// Attempt number, starting at one.
    pub attempt: u32,
    /// Recovery reason.
    pub reason: String,
    /// Recovery policy in force.
    pub policy: RecoveryPolicy,
}
