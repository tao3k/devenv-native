//! Public runtime instance construction and state shells.

use crate::error::Result;
use crate::ir::{BpmnPackage, ProcessKey};
use crate::ir_index_api::BpmnNodeIndex;
use crate::runtime::{
    JoinRuntimeState, PendingHostWork, PendingHostWorkKind, TokenRecord, WaitRegistration,
};
use crate::runtime_repeat_api::{
    ParallelMultiInstanceState, SequentialMultiInstanceState, StandardLoopState,
};
use serde::{Deserialize, Deserializer};
use std::borrow::Borrow;
use std::sync::Arc;

/// Initial values required to create one workflow instance shell.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BpmnInstanceInit {
    /// Stable instance identifier.
    pub instance_id: Arc<str>,
    /// Initial workflow variables.
    pub initial_variables: serde_json::Value,
    /// Initial timestamp in Unix milliseconds.
    pub initial_timestamp_ms: u64,
}

impl BpmnInstanceInit {
    /// Creates instance-initialization data.
    #[must_use]
    pub fn new(
        instance_id: impl AsRef<str>,
        initial_variables: serde_json::Value,
        initial_timestamp_ms: u64,
    ) -> Self {
        Self {
            instance_id: Arc::<str>::from(instance_id.as_ref()),
            initial_variables,
            initial_timestamp_ms,
        }
    }
}

/// High-level node runtime status shell.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeRuntimeStatus {
    /// Initial dormant state.
    #[default]
    Idle,
    /// Runnable or enqueued state.
    Queued,
    /// External or host-side execution in progress.
    Executing,
    /// Finished state.
    Completed,
    /// Cancelled by an interrupting boundary event.
    Cancelled,
    /// Terminal failure state.
    Failed,
}

/// Per-node runtime record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct NodeRuntimeState {
    /// Node index.
    pub node_index: BpmnNodeIndex,
    /// Current runtime status.
    pub status: NodeRuntimeStatus,
}

/// Runtime trace event discriminator.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnExecutionTraceEventKind {
    /// A node status changed.
    NodeStatus,
    /// A runtime token traversed one BPMN sequence flow.
    FlowTake,
}

/// Ordered BPMN execution trace event.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnExecutionTraceEvent {
    /// Monotonic trace sequence inside the instance.
    pub sequence: u64,
    /// Process identity active when the event was recorded.
    pub process: ProcessKey,
    /// Event discriminator.
    pub kind: BpmnExecutionTraceEventKind,
    /// Node index for node-status events and flow target nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_index: Option<BpmnNodeIndex>,
    /// Edge index for sequence-flow traversal events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_index: Option<u32>,
    /// Runtime status for node-status events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<NodeRuntimeStatus>,
}

/// Durable lifecycle event discriminator for checkpointed human-task work.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnHumanTaskLifecycleEventKind {
    /// A `userTask` or `manualTask` pending host-work item was created.
    Created,
    /// A previously unclaimed human task was claimed.
    Claimed,
    /// A claimed human task was released back to the unclaimed worklist.
    Released,
    /// A human task completion passed validation and was applied.
    Completed,
}

/// Durable checkpointed lifecycle event for BPMN `userTask` and `manualTask`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskLifecycleEvent {
    /// Monotonic sequence inside the human-task lifecycle ledger.
    pub sequence: u64,
    /// Human-task lifecycle event kind.
    pub kind: BpmnHumanTaskLifecycleEventKind,
    /// Unix timestamp in milliseconds when the event was recorded.
    pub occurred_at_ms: u64,
    /// BPMN process identifier that owns the human task.
    pub process_id: String,
    /// Stable BPMN activity identifier for the human task.
    pub activity_id: String,
    /// Runtime token identifier for the human task.
    pub token_id: u64,
    /// BPMN node index for the human task.
    pub node_index: BpmnNodeIndex,
    /// Human work kind.
    pub work_kind: PendingHostWorkKind,
    /// Optional host- or operator-facing claimant identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimant: Option<String>,
    /// Optional host-generated work identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,
}

/// High-level instance lifecycle shell.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceLifecycle {
    /// Ready to execute.
    Ready,
    /// Currently progressing.
    Running,
    /// Waiting on external or host-side progress.
    Waiting,
    /// Suspended intentionally.
    Suspended,
    /// Completed successfully.
    Completed,
    /// Failed terminally.
    Failed,
}

/// Suspend reason shell.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuspendReason {
    /// Suspended by host request.
    HostRequested,
    /// Suspended due to external wait boundary.
    ExternalWait,
    /// Suspended because the runtime reached a reserved DMN placeholder task.
    DmnPlaceholder,
    /// Suspended by scaffold-only flow control.
    ScaffoldBoundary,
}

/// Snapshot of one active event-competition owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventCompetitionState {
    /// Owning gateway node index.
    pub gateway_node_index: BpmnNodeIndex,
    /// Candidate waiting node indices owned by the gateway.
    pub wait_node_indices: Vec<BpmnNodeIndex>,
}

/// How the active compensation queue should resume once all handlers finish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransactionCompensationCompletionMode {
    /// Resume by routing through the parent cancel boundary.
    #[default]
    CancelBoundary,
    /// Resume by completing the current scope normally.
    ScopeCompletion,
    /// Resume by routing from one completed intermediate throw-compensation node.
    IntermediateRouting { node_index: BpmnNodeIndex },
    /// Let the queue drain independently while another token continues routing.
    Detached,
}

/// Bounded transaction compensation queue state.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct TransactionCompensationState {
    /// Completed compensable activity node indices in completion order.
    #[serde(default)]
    pub completed_activity_node_indices: Vec<BpmnNodeIndex>,
    /// Pending compensation handler node indices in execution order.
    #[serde(default)]
    pub pending_handler_node_indices: Vec<BpmnNodeIndex>,
    /// Whether the compensation queue is currently running.
    #[serde(default)]
    pub cancelling: bool,
    /// How the current scope should resume after the queue drains.
    #[serde(default)]
    pub completion_mode: TransactionCompensationCompletionMode,
}

/// Detached transaction compensation queue that can outlive one child frame.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DetachedTransactionCompensationState {
    /// Child process identity that owns the detached compensation handlers.
    pub process: ProcessKey,
    /// Cached child process position inside the owning package.
    #[serde(default)]
    pub process_index: u32,
    /// Remaining compensation handlers in reverse execution order so `pop()`
    /// yields the next handler deterministically.
    #[serde(default)]
    pub pending_handler_node_indices: Vec<BpmnNodeIndex>,
}

/// Snapshot of one parent execution frame while a bounded call activity runs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CallActivityFrame {
    /// Suspended parent process identity.
    pub process: ProcessKey,
    /// Cached parent process position inside the owning package.
    #[serde(default)]
    pub process_index: u32,
    /// Parent call-activity node that should resume after the child process completes.
    pub return_node_index: BpmnNodeIndex,
    /// Suspended parent node runtime state.
    pub node_states: Vec<NodeRuntimeState>,
    /// Suspended parent active runtime tokens.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_tokens: Vec<TokenRecord>,
    /// Suspended parent join progress records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub joins: Vec<JoinRuntimeState>,
    /// Suspended parent standard-loop progress records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub standard_loops: Vec<StandardLoopState>,
    /// Suspended parent sequential multi-instance owner records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sequential_multi_instances: Vec<SequentialMultiInstanceState>,
    /// Suspended parent parallel multi-instance owner records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parallel_multi_instances: Vec<ParallelMultiInstanceState>,
    /// Suspended parent waiting registrations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waits: Vec<WaitRegistration>,
    /// Suspended parent event-competition state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_competition: Option<EventCompetitionState>,
    /// Suspended parent pending host work references.
    #[serde(
        default,
        deserialize_with = "deserialize_pending_host_work_collection",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub pending_host_work: Vec<PendingHostWork>,
    /// Suspended parent suspend reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suspend_reason: Option<SuspendReason>,
    /// Optional variable snapshot restored when a transaction shell cancels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_cancel_variables: Option<serde_json::Value>,
    /// Optional bounded compensation queue state for one transaction shell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_compensation: Option<TransactionCompensationState>,
}

/// Mutable BPMN instance state shell.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BpmnInstanceState {
    /// Stable instance identifier.
    pub instance_id: Arc<str>,
    /// Process identity.
    pub process: ProcessKey,
    /// Cached process position inside the owning package.
    ///
    /// Runtime resolution validates and repairs this index against
    /// `process_id`, so checkpoints written before this field existed remain
    /// recoverable.
    #[serde(default)]
    pub process_index: u32,
    /// Suspended parent frames while bounded call activities run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub call_stack: Vec<CallActivityFrame>,
    /// Monotonic checkpoint sequence.
    pub sequence: u64,
    /// Next token id reserved for this instance lifetime.
    ///
    /// Older checkpoints that lack this field recover the cursor from live
    /// and suspended token-bearing state before the next allocation.
    #[serde(default)]
    pub next_token_id: u64,
    /// High-level lifecycle state.
    pub lifecycle: InstanceLifecycle,
    /// Workflow variables.
    pub variables: serde_json::Value,
    /// Per-node runtime state.
    pub node_states: Vec<NodeRuntimeState>,
    /// Active runtime tokens.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_tokens: Vec<TokenRecord>,
    /// Ordered runtime execution trace.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trace: Vec<BpmnExecutionTraceEvent>,
    /// Join progress records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub joins: Vec<JoinRuntimeState>,
    /// Active standard-loop progress records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub standard_loops: Vec<StandardLoopState>,
    /// Active sequential multi-instance owner records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sequential_multi_instances: Vec<SequentialMultiInstanceState>,
    /// Active parallel multi-instance owner records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parallel_multi_instances: Vec<ParallelMultiInstanceState>,
    /// Waiting registrations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waits: Vec<WaitRegistration>,
    /// Active event-competition state for multi-wait ownership.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_competition: Option<EventCompetitionState>,
    /// Detached compensation handlers that continue after a transaction end
    /// event resumes its parent scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detached_transaction_compensation: Option<DetachedTransactionCompensationState>,
    /// Pending host work references.
    #[serde(
        default,
        deserialize_with = "deserialize_pending_host_work_collection",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub pending_host_work: Vec<PendingHostWork>,
    /// Durable human-task lifecycle ledger for `userTask` and `manualTask`.
    pub human_task_events: Vec<BpmnHumanTaskLifecycleEvent>,
    /// Optional suspend reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suspend_reason: Option<SuspendReason>,
    /// Last update timestamp in Unix milliseconds.
    pub updated_at_ms: u64,
}

/// Creates a workflow instance shell for one process.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingProcess`] when the target process does not
/// exist in the provided package.
pub fn create_instance(
    package: impl Borrow<BpmnPackage>,
    process_id: &str,
    init: BpmnInstanceInit,
) -> Result<BpmnInstanceState> {
    crate::runtime::create_instance_impl(package, process_id, init)
}

fn deserialize_pending_host_work_collection<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<PendingHostWork>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PendingHostWorkField {
        None(Option<PendingHostWork>),
        One(PendingHostWork),
        Many(Vec<PendingHostWork>),
    }

    let value = Option::<PendingHostWorkField>::deserialize(deserializer)?;
    Ok(match value {
        None | Some(PendingHostWorkField::None(None)) => Vec::new(),
        Some(PendingHostWorkField::None(Some(pending)) | PendingHostWorkField::One(pending)) => {
            vec![pending]
        }
        Some(PendingHostWorkField::Many(pending)) => pending,
    })
}
