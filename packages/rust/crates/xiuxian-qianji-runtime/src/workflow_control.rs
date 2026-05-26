//! Runtime-owned workflow-control port for durable worker loops.
//!
//! This module defines the dependency-safe seam between worker loops and the
//! workflow-control service that owns BPMN checkpoint replay. The runtime crate
//! owns the request shapes and trait; concrete services implement the port
//! from higher-level crates.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_json::Value;
use xiuxian_qianji_bpmn_engine::{BpmnHostBridge, PendingHostWork, PendingHostWorkKind};

use crate::{QianjiRuntimeBpmnActivityId, QianjiRuntimeBpmnProcessId, QianjiRuntimeBpmnTokenId};

/// Owned runtime BPMN workflow instance identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QianjiRuntimeBpmnInstanceId(String);

impl QianjiRuntimeBpmnInstanceId {
    /// Creates an owned runtime BPMN workflow instance id.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrows the serialized workflow instance id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the owned serialized workflow instance id.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for QianjiRuntimeBpmnInstanceId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Runtime BPMN source path used by checkpoint-backed workflow operations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QianjiRuntimeBpmnSourcePath(PathBuf);

impl QianjiRuntimeBpmnSourcePath {
    /// Creates a runtime BPMN source path.
    #[must_use]
    pub fn new(value: impl Into<PathBuf>) -> Self {
        Self(value.into())
    }

    /// Borrows the BPMN source path.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    /// Returns the owned BPMN source path.
    #[must_use]
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

/// Runtime DMN source path collection loaded alongside a BPMN source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeDmnSourcePaths(Vec<PathBuf>);

impl QianjiRuntimeDmnSourcePaths {
    /// Creates a runtime DMN source path collection.
    #[must_use]
    pub fn new(value: impl Into<Vec<PathBuf>>) -> Self {
        Self(value.into())
    }

    /// Creates an empty runtime DMN source path collection.
    #[must_use]
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// Borrows the DMN source paths.
    #[must_use]
    pub fn as_slice(&self) -> &[PathBuf] {
        self.0.as_slice()
    }

    /// Returns the owned DMN source paths.
    #[must_use]
    pub fn into_vec(self) -> Vec<PathBuf> {
        self.0
    }
}

/// Runtime flag for continuing through non-human host work after completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QianjiRuntimeContinueUntilHumanBoundary(bool);

impl QianjiRuntimeContinueUntilHumanBoundary {
    /// Creates a runtime continuation flag.
    #[must_use]
    pub const fn new(value: bool) -> Self {
        Self(value)
    }

    /// Returns the raw continuation flag.
    #[must_use]
    pub const fn as_bool(self) -> bool {
        self.0
    }
}

/// Pending host-work result kind accepted by runtime workflow completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QianjiRuntimeWorkflowTaskCompletionKind {
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

/// Runtime payload for completing one checkpoint-backed pending host task.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiRuntimeWorkflowTaskCompletionPayload {
    /// Runtime token identifier for the pending host work.
    pub token_id: QianjiRuntimeBpmnTokenId,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: QianjiRuntimeBpmnProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: QianjiRuntimeBpmnActivityId,
    /// Pending host-work result kind.
    pub kind: QianjiRuntimeWorkflowTaskCompletionKind,
    /// Payload merged into workflow variables.
    pub data: Value,
    /// Optional claimant supplied by the host when completing claimed human work.
    pub claimant: Option<String>,
}

/// Runtime request for loading checkpoint-backed workflow status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeWorkflowStatusRequest<C> {
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: QianjiRuntimeBpmnInstanceId,
    /// Checkpoint backend to inspect for this bounded status request.
    pub checkpoint_backend: C,
}

/// Runtime request for preparing a checkpoint-backed workflow resume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeWorkflowResumeRequest<C> {
    /// BPMN source path used by the checkpoint-backed workflow.
    pub bpmn_source: QianjiRuntimeBpmnSourcePath,
    /// DMN sources loaded alongside the BPMN package.
    pub dmn_sources: QianjiRuntimeDmnSourcePaths,
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: QianjiRuntimeBpmnInstanceId,
    /// Checkpoint backend that owns persisted workflow state.
    pub checkpoint_backend: C,
}

/// Runtime request for completing a prepared checkpoint-backed workflow task.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiRuntimeWorkflowTaskCompleteRequest<C> {
    /// BPMN source path used by the checkpoint-backed workflow.
    pub bpmn_source: QianjiRuntimeBpmnSourcePath,
    /// DMN sources loaded alongside the BPMN package.
    pub dmn_sources: QianjiRuntimeDmnSourcePaths,
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: QianjiRuntimeBpmnInstanceId,
    /// Checkpoint backend that owns persisted workflow state.
    pub checkpoint_backend: C,
    /// Explicit completion payload for the pending host task.
    pub completion: QianjiRuntimeWorkflowTaskCompletionPayload,
    /// Whether execution should continue through non-human host tasks.
    pub continue_until_human_boundary: QianjiRuntimeContinueUntilHumanBoundary,
}

impl<C: Clone> QianjiRuntimeWorkflowTaskCompleteRequest<C> {
    /// Builds the resume request required before completing this host task.
    #[must_use]
    pub fn workflow_resume_request(&self) -> QianjiRuntimeWorkflowResumeRequest<C> {
        QianjiRuntimeWorkflowResumeRequest {
            bpmn_source: self.bpmn_source.clone(),
            dmn_sources: self.dmn_sources.clone(),
            instance_id: self.instance_id.clone(),
            checkpoint_backend: self.checkpoint_backend.clone(),
        }
    }
}

/// Runtime status view needed by worker loops.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiRuntimeWorkflowStatusView {
    /// Checkpoint-derived pending host work in workflow order.
    pub pending_host_work: Vec<PendingHostWork>,
}

impl QianjiRuntimeWorkflowStatusView {
    /// Creates a runtime status view from checkpoint-derived pending host work.
    #[must_use]
    pub fn new(pending_host_work: Vec<PendingHostWork>) -> Self {
        Self { pending_host_work }
    }

    /// Returns the number of checkpoint-derived pending host-work items.
    #[must_use]
    pub fn pending_host_work_count(&self) -> usize {
        self.pending_host_work.len()
    }

    /// Returns the first pending host-work item matching the requested kind.
    #[must_use]
    pub fn first_pending_host_work_by_kind(
        &self,
        kind: &PendingHostWorkKind,
    ) -> Option<&PendingHostWork> {
        self.pending_host_work
            .iter()
            .find(|work| &work.kind == kind)
    }

    /// Consumes the status view and returns the first matching pending host-work item.
    #[must_use]
    pub fn into_first_pending_host_work_by_kind(
        self,
        kind: &PendingHostWorkKind,
    ) -> Option<PendingHostWork> {
        self.pending_host_work
            .into_iter()
            .find(|work| &work.kind == kind)
    }
}

/// Port implemented by services that can drive checkpoint-backed workflows.
#[async_trait]
pub trait QianjiRuntimeWorkflowControlPort<H>: Send + Sync
where
    H: BpmnHostBridge + Send + Sync,
{
    /// Checkpoint backend selector owned by the concrete service.
    type CheckpointBackend: Clone + Send + Sync + 'static;
    /// Prepared workflow resume object owned by the concrete service.
    type PreparedResume: Send;
    /// Task-completion report owned by the concrete service.
    type TaskCompleteReport: Clone + Send;
    /// Concrete error type returned by the control service.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Loads the current workflow status view from durable checkpoint state.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the concrete service cannot resolve the
    /// checkpoint backend, load the workflow checkpoint, or shape pending
    /// host-work facts.
    async fn load_workflow_status_view(
        &self,
        request: QianjiRuntimeWorkflowStatusRequest<Self::CheckpointBackend>,
    ) -> Result<QianjiRuntimeWorkflowStatusView, Self::Error>;

    /// Prepares a checkpoint-backed workflow resume for later completion.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the concrete service cannot resolve
    /// sources, load packages, load checkpoint state, or prepare replay inputs.
    async fn prepare_resume_workflow(
        &self,
        request: QianjiRuntimeWorkflowResumeRequest<Self::CheckpointBackend>,
    ) -> Result<Self::PreparedResume, Self::Error>;

    /// Completes pending host work against an already prepared workflow.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the concrete service rejects completion,
    /// cannot resume the prepared workflow, or cannot persist resulting
    /// checkpoint state.
    async fn complete_prepared_workflow_task_until_host_boundary(
        &self,
        prepared: Self::PreparedResume,
        request: QianjiRuntimeWorkflowTaskCompleteRequest<Self::CheckpointBackend>,
        host: &H,
    ) -> Result<Self::TaskCompleteReport, Self::Error>;
}
