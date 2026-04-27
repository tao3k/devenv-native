use crate::bpmn::error::BpmnOrchestrationError;
use crate::runtime_config::QianjiRuntimeEnv;
use crate::scheduler_identity::SchedulerAgentIdentity;
use std::io;

#[path = "service_api/api.rs"]
mod service;
#[path = "types/api.rs"]
mod types;

pub use types::{
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnPreparedWorkflowStart,
    QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowCancelRequest,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowEventPollReport,
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowInstanceSummary,
    QianjiBpmnWorkflowInstancesReport, QianjiBpmnWorkflowInstancesRequest,
    QianjiBpmnWorkflowInterruptReport, QianjiBpmnWorkflowInterruptRequest,
    QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowStartReport,
    QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowStatusReport,
    QianjiBpmnWorkflowStatusRequest, QianjiBpmnWorkflowTaskCompleteReport,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload,
};

/// Error returned by the lib-owned BPMN workflow control service.
#[derive(Debug, thiserror::Error)]
pub enum QianjiBpmnWorkflowControlError {
    /// Returned when filesystem, path-resolution, or runtime-config lookup
    /// fails while preparing a workflow run.
    #[error("BPMN workflow control I/O error: {0}")]
    Io(#[from] io::Error),
    /// Returned when the requested BPMN checkpoint is missing.
    #[error("BPMN workflow checkpoint not found for instance `{instance_id}`")]
    CheckpointMissing {
        /// Workflow instance identifier that could not be loaded.
        instance_id: String,
    },
    /// Returned when BPMN package loading or workflow execution fails inside
    /// the host-owned orchestration facade.
    #[error(transparent)]
    Orchestration(#[from] BpmnOrchestrationError),
}

/// Lib-owned BPMN workflow control service that prepares one bounded
/// package/checkpoint request and dispatches it through the lower-level
/// execution facade.
#[derive(Debug, Clone, Default)]
pub struct QianjiBpmnWorkflowControlService {
    pub(crate) runtime_env: Option<QianjiRuntimeEnv>,
    pub(crate) scheduler_identity: Option<SchedulerAgentIdentity>,
}

impl QianjiBpmnWorkflowControlService {
    /// Creates a workflow control service with default runtime resolution.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Installs one explicit runtime environment override for checkpoint
    /// config resolution.
    #[must_use]
    pub fn with_runtime_env(mut self, runtime_env: QianjiRuntimeEnv) -> Self {
        self.runtime_env = Some(runtime_env);
        self
    }

    /// Installs one scheduler identity that may enable scheduler-owned BPMN
    /// lifecycle behavior on the Valkey-backed path.
    #[must_use]
    pub fn with_scheduler_identity(mut self, scheduler_identity: SchedulerAgentIdentity) -> Self {
        self.scheduler_identity = Some(scheduler_identity);
        self
    }
}
