//! Execution request and report contracts for BPMN workflow control.

use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::driver::{QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest};
use crate::bpmn::identity::{
    QianjiBpmnActivityId, QianjiBpmnProcessId, QianjiBpmnStartAtNodeId,
    QianjiBpmnWorkflowInstanceId,
};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{BpmnCheckpointEnvelope, BpmnPackage};

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
    pub process_id: QianjiBpmnProcessId,
    /// Workflow instance identifier used for checkpoint lookup and fresh runs.
    pub instance_id: QianjiBpmnWorkflowInstanceId,
    /// Optional initial variables for a fresh run.
    pub initial_variables: Option<Value>,
    /// Optional node id for a fresh synthetic start-at run.
    pub start_at_node_id: Option<QianjiBpmnStartAtNodeId>,
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
    /// Checkpoint envelope loaded while preparing a resume request.
    ///
    /// Fresh starts leave this empty. Prepared resume paths may pass this into
    /// the execution driver to avoid loading the same checkpoint twice inside
    /// one bounded operation.
    pub loaded_checkpoint: Option<BpmnCheckpointEnvelope>,
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
    pub instance_id: QianjiBpmnWorkflowInstanceId,
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
    pub instance_id: QianjiBpmnWorkflowInstanceId,
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
    pub process_id: QianjiBpmnProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: QianjiBpmnActivityId,
    /// Pending host-work result kind.
    pub kind: QianjiBpmnWorkflowTaskCompletionKind,
    /// User- or operator-supplied payload merged into workflow variables.
    pub data: serde_json::Value,
    /// Optional claimant supplied by the host when completing claimed human
    /// work.
    pub claimant: Option<String>,
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
    pub instance_id: QianjiBpmnWorkflowInstanceId,
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

/// Typed request for completing multiple pending host work items from one
/// checkpoint-backed BPMN workflow host boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiBpmnWorkflowTaskCompleteBatchRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    pub dmn_paths: Vec<PathBuf>,
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: QianjiBpmnWorkflowInstanceId,
    /// Checkpoint backend that already owns persisted workflow state.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    /// Explicit completion payloads for pending host tasks.
    pub completions: Vec<QianjiBpmnWorkflowTaskCompletionPayload>,
}

/// Report returned by the workflow control service after a host-task
/// completion batch.
pub type QianjiBpmnWorkflowTaskCompleteBatchReport = QianjiBpmnWorkflowResumeReport;
