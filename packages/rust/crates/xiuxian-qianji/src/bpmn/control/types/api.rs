use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::driver::{QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest};
use qianji_bpmn_engine::{BpmnInstanceState, BpmnPackage};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

/// Checkpoint backend selection for BPMN workflow control surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QianjiBpmnWorkflowCheckpointBackend {
    /// Resolve the runtime-configured Valkey checkpoint backend.
    RuntimeValkey,
    /// Use the configured local `DuckDB` workflow-state store when no server is running.
    #[cfg(feature = "duckdb")]
    LocalDuckDb,
}

/// Typed request for starting or resuming one bounded BPMN workflow instance.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiBpmnWorkflowStartRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    pub dmn_paths: Vec<PathBuf>,
    /// BPMN process identifier used for a fresh run.
    pub process_id: String,
    /// Workflow instance identifier used for checkpoint lookup and fresh runs.
    pub instance_id: String,
    /// Optional initial variables for a fresh run.
    pub initial_variables: Option<Value>,
    /// Optional node id for a fresh synthetic start-at run.
    pub start_at_node_id: Option<String>,
    /// Optional checkpoint backend to use for this bounded run.
    pub checkpoint_backend: Option<QianjiBpmnWorkflowCheckpointBackend>,
}

/// Prepared workflow-start inputs resolved by the control service before host
/// construction or execution begins.
#[derive(Debug, Clone)]
pub struct QianjiBpmnPreparedWorkflowStart {
    /// Loaded BPMN package shared with the subsequent execution phase.
    pub package: Arc<BpmnPackage>,
    /// Resolved BPMN source path rooted against the current working directory.
    pub resolved_bpmn_path: PathBuf,
    /// Resolved DMN source paths rooted against the current working directory.
    pub resolved_dmn_paths: Vec<PathBuf>,
    /// Resolved checkpoint store for this bounded run, if any.
    pub checkpoint_store: Option<QianjiBpmnCheckpointStore>,
    /// Engine-facing execution request shaped from the typed workflow request.
    pub execution_request: QianjiBpmnExecutionRequest,
}

/// Report returned by the workflow control service after one bounded run.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowStartReport {
    /// Resolved BPMN source path rooted against the current working directory.
    pub resolved_bpmn_path: PathBuf,
    /// Resolved DMN source paths rooted against the current working directory.
    pub resolved_dmn_paths: Vec<PathBuf>,
    /// Resolved checkpoint store for this bounded run, if any.
    pub checkpoint_store: Option<QianjiBpmnCheckpointStore>,
    /// Bounded execution outcome emitted by the lower-level BPMN facade.
    pub execution: QianjiBpmnExecutionReport,
}

/// Typed request for resuming one checkpoint-backed BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowResumeRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    pub dmn_paths: Vec<PathBuf>,
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: String,
    /// Checkpoint backend that already owns persisted workflow state.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Prepared workflow-resume inputs resolved by the control service before host
/// construction or execution begins.
pub type QianjiBpmnPreparedWorkflowResume = QianjiBpmnPreparedWorkflowStart;

/// Report returned by the workflow control service after one resumed bounded
/// run.
pub type QianjiBpmnWorkflowResumeReport = QianjiBpmnWorkflowStartReport;

/// Typed request for polling external events on one checkpoint-backed BPMN
/// workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowEventPollRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    pub dmn_paths: Vec<PathBuf>,
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: String,
    /// Checkpoint backend that already owns persisted workflow state.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Report returned by the workflow control service after one external-event
/// poll action.
pub type QianjiBpmnWorkflowEventPollReport = QianjiBpmnWorkflowResumeReport;

/// Host-work result kind accepted by explicit task completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QianjiBpmnWorkflowTaskCompletionKind {
    /// Complete a BPMN `sendTask`.
    Send,
    /// Complete a BPMN `serviceTask`.
    Service,
    /// Complete a BPMN `scriptTask`.
    Script,
    /// Complete a BPMN `userTask`.
    User,
    /// Complete a BPMN `manualTask`.
    Manual,
}

/// Explicit payload for completing pending host work on one checkpoint-backed
/// BPMN workflow instance.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiBpmnWorkflowTaskCompletionPayload {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: String,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: String,
    /// Pending host-work result kind.
    pub kind: QianjiBpmnWorkflowTaskCompletionKind,
    /// User- or operator-supplied payload merged into workflow variables.
    pub data: serde_json::Value,
}

/// Typed request for completing pending host work on one checkpoint-backed BPMN
/// workflow instance.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiBpmnWorkflowTaskCompleteRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    pub dmn_paths: Vec<PathBuf>,
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: String,
    /// Checkpoint backend that already owns persisted workflow state.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    /// Explicit completion payload for the pending host task.
    pub completion: QianjiBpmnWorkflowTaskCompletionPayload,
    /// Continue through fixture-backed non-human host tasks until the next
    /// user/manual boundary after applying `completion`.
    pub continue_until_human_boundary: bool,
}

/// Report returned by the workflow control service after one host-task
/// completion action.
pub type QianjiBpmnWorkflowTaskCompleteReport = QianjiBpmnWorkflowResumeReport;

/// Typed request for loading one checkpoint-backed BPMN workflow status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowStatusRequest {
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: String,
    /// Checkpoint backend to inspect for this bounded status request.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Typed request for listing checkpoint-backed BPMN workflow instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowInstancesRequest {
    /// Checkpoint backend to inspect for this bounded instance-list request.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Typed request for canceling one checkpoint-backed BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowCancelRequest {
    /// Workflow instance identifier used for checkpoint lookup and deletion.
    pub instance_id: String,
    /// Checkpoint backend to cancel for this bounded workflow instance.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Typed request for interrupting one checkpoint-backed BPMN workflow instance
/// while preserving durable checkpoint state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowInterruptRequest {
    /// Workflow instance identifier used for checkpoint lookup and preservation.
    pub instance_id: String,
    /// Checkpoint backend to interrupt for this bounded workflow instance.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Report returned by the workflow control service after one checkpoint-first
/// BPMN workflow status load.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowStatusReport {
    /// Resolved checkpoint store used for this status request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Monotonic checkpoint sequence loaded from the persisted envelope.
    pub checkpoint_sequence: u64,
    /// Durable BPMN instance state stored in the checkpoint payload.
    pub instance: BpmnInstanceState,
}

/// Compact checkpoint summary for one persisted BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowInstanceSummary {
    /// Workflow instance identifier.
    pub instance_id: String,
    /// BPMN process identifier.
    pub process_id: String,
    /// BPMN package identifier.
    pub package_id: String,
    /// Durable instance lifecycle.
    pub lifecycle: qianji_bpmn_engine::InstanceLifecycle,
    /// Monotonic checkpoint sequence loaded from the persisted envelope.
    pub checkpoint_sequence: u64,
    /// Engine state sequence inside the checkpoint payload.
    pub state_sequence: u64,
    /// Last checkpoint update timestamp in unix milliseconds.
    pub updated_at_ms: u64,
    /// Number of active runtime tokens.
    pub active_token_count: usize,
    /// Number of pending host-work entries.
    pub pending_host_work_count: usize,
    /// Number of registered waits.
    pub wait_registration_count: usize,
}

impl QianjiBpmnWorkflowInstanceSummary {
    pub(crate) fn from_checkpoint(checkpoint: qianji_bpmn_engine::BpmnCheckpointEnvelope) -> Self {
        Self {
            instance_id: checkpoint.state.instance_id.as_ref().to_string(),
            process_id: checkpoint.state.process.process_id.as_ref().to_string(),
            package_id: checkpoint.state.process.package_id.as_ref().to_string(),
            lifecycle: checkpoint.state.lifecycle,
            checkpoint_sequence: checkpoint.sequence,
            state_sequence: checkpoint.state.sequence,
            updated_at_ms: checkpoint.state.updated_at_ms,
            active_token_count: checkpoint.state.active_tokens.len(),
            pending_host_work_count: checkpoint.state.pending_host_work.len(),
            wait_registration_count: checkpoint.state.waits.len(),
        }
    }
}

/// Report returned by the workflow control service after listing checkpointed
/// BPMN workflow instances.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowInstancesReport {
    /// Resolved checkpoint store used for this instance-list request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Latest known checkpoint summaries, newest first when supported by the backend.
    pub instances: Vec<QianjiBpmnWorkflowInstanceSummary>,
}

/// Report returned by the workflow control service after one checkpoint-first
/// BPMN workflow cancellation.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowCancelReport {
    /// Resolved checkpoint store used for this cancel request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Monotonic checkpoint sequence loaded before deletion.
    pub checkpoint_sequence: u64,
    /// Durable BPMN instance state loaded before deletion.
    pub instance: BpmnInstanceState,
}

/// Report returned by the workflow control service after one checkpoint-first
/// BPMN workflow interruption.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowInterruptReport {
    /// Resolved checkpoint store used for this interrupt request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Monotonic checkpoint sequence persisted after interruption.
    pub checkpoint_sequence: u64,
    /// Durable BPMN instance state persisted after interruption.
    pub instance: BpmnInstanceState,
}
