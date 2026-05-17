//! Core control-plane data model.

use crate::{ArtifactId, ArtifactKind, EvidenceId, GateName, LeaseId, RunId, StepId, WorkerId};

/// Run lifecycle status reconstructed from control events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// Intent captured but not yet admitted.
    #[default]
    Draft,
    /// Run admitted by the control planner.
    Admitted,
    /// Runnable work has been planned.
    Planned,
    /// At least one step is actively running.
    Running,
    /// Run is waiting on a tool, worker, or human boundary.
    Waiting,
    /// Recovery is in progress.
    Recovering,
    /// Run is blocked by a deterministic gate or missing prerequisite.
    Blocked,
    /// Run completed successfully.
    Completed,
    /// Run failed terminally.
    Failed,
    /// Run was intentionally aborted.
    Aborted,
}

/// Step lifecycle status reconstructed from control events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// Step declared but not yet queued.
    #[default]
    Pending,
    /// Step is waiting in a hot-state queue.
    Queued,
    /// Step has an active lease.
    Leased,
    /// Step execution has started.
    Running,
    /// Step is waiting on an external boundary.
    Waiting,
    /// Step is in recovery.
    Recovering,
    /// Step succeeded.
    Succeeded,
    /// Step failed.
    Failed,
    /// Step is blocked by a deterministic condition.
    Blocked,
    /// Step was cancelled.
    Cancelled,
}

/// Why a run or step is waiting.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitReason {
    /// Waiting on tool output.
    Tool,
    /// Waiting on human approval or input.
    Human,
    /// Waiting on worker capacity.
    Worker,
    /// Waiting on external IO or service response.
    External,
}

/// Optional budget for one run or step.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Budget {
    /// Maximum wall time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_time_ms: Option<u64>,
    /// Maximum model or provider tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u64>,
    /// Maximum cost in USD micros.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_usd_micros: Option<u64>,
}

/// One observed cost row.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CostObservation {
    /// Provider or tool name.
    pub provider: String,
    /// Optional model id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Prompt tokens.
    #[serde(default)]
    pub prompt_tokens: u64,
    /// Completion tokens.
    #[serde(default)]
    pub completion_tokens: u64,
    /// Total tokens when provider reports a direct value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
    /// Cost in USD micros.
    #[serde(default)]
    pub cost_usd_micros: u64,
    /// Provider/tool latency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl CostObservation {
    /// Returns total tokens, deriving the value when needed.
    #[must_use]
    pub fn observed_total_tokens(&self) -> u64 {
        self.total_tokens
            .unwrap_or(self.prompt_tokens + self.completion_tokens)
    }
}

/// Evidence attached to a step.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EvidenceRef {
    /// Evidence id.
    pub evidence_id: EvidenceId,
    /// Optional required-evidence key this ref satisfies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirement_key: Option<String>,
    /// Human-readable source.
    pub source: String,
    /// Optional URI, file path, or artifact route.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Short summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Artifact attached to a run or step.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArtifactRef {
    /// Artifact id.
    pub artifact_id: ArtifactId,
    /// Logical artifact kind.
    pub artifact_kind: ArtifactKind,
    /// URI, file path, or external route.
    pub uri: String,
    /// Optional content digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_digest: Option<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Gate evaluation result.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GateResult {
    /// Stable gate name.
    pub gate_name: GateName,
    /// Whether the gate passed.
    pub passed: bool,
    /// Whether required evidence coverage is complete.
    pub required_evidence_covered: bool,
    /// Required evidence keys covered by selected evidence.
    #[serde(default)]
    pub selected_required_evidence: Vec<String>,
    /// Required evidence keys still missing.
    #[serde(default)]
    pub missing_required_evidence: Vec<String>,
    /// Human-readable reasons.
    #[serde(default)]
    pub reasons: Vec<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Recovery policy declared for a run or step.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecoveryPolicy {
    /// Maximum attempts.
    pub max_attempts: u32,
    /// Backoff before retry.
    #[serde(default)]
    pub backoff_ms: u64,
    /// Whether human approval is required before retry.
    #[serde(default)]
    pub require_human_approval: bool,
}

/// Runnable step entry for hot-state queues.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunnableStep {
    /// Owning run id.
    pub run_id: RunId,
    /// Step id.
    pub step_id: StepId,
    /// Higher values are acquired first.
    #[serde(default)]
    pub priority: i64,
    /// Earliest acquisition time.
    #[serde(default)]
    pub not_before_ms: u64,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Worker requesting hot-state leases.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerRef {
    /// Worker id.
    pub worker_id: WorkerId,
    /// Worker capabilities.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Active step lease.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StepLease {
    /// Lease id.
    pub lease_id: LeaseId,
    /// Owning run id.
    pub run_id: RunId,
    /// Leased step id.
    pub step_id: StepId,
    /// Owning worker id.
    pub worker_id: WorkerId,
    /// Acquisition timestamp.
    pub acquired_at_ms: u64,
    /// Expiry timestamp.
    pub expires_at_ms: u64,
}

impl StepLease {
    /// Returns true when the lease is still valid at `now_ms`.
    #[must_use]
    pub const fn is_active_at(&self, now_ms: u64) -> bool {
        now_ms < self.expires_at_ms
    }
}

/// Worker heartbeat stored in hot state.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerHeartbeat {
    /// Worker id.
    pub worker_id: WorkerId,
    /// Last heartbeat timestamp.
    pub observed_at_ms: u64,
    /// Heartbeat expiry timestamp.
    pub expires_at_ms: u64,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}
