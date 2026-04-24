use super::control_service as service;
use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::driver::{QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest};
use crate::bpmn::error::BpmnOrchestrationError;
use crate::bpmn::session::QianjiBpmnSession;
use crate::runtime_config::QianjiRuntimeEnv;
use crate::scheduler::SchedulerAgentIdentity;
use qianji_bpmn_engine::BpmnExecutionTraceEvent;
use qianji_bpmn_engine::BpmnHostBridge;
use qianji_bpmn_engine::BpmnInstanceState;
use qianji_bpmn_engine::BpmnPackage;
use serde_json::Value;
use std::io;
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

/// Typed request for completing pending host work on one checkpoint-backed BPMN
/// workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowTaskCompleteRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    pub dmn_paths: Vec<PathBuf>,
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: String,
    /// Checkpoint backend that already owns persisted workflow state.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
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

/// Typed request for canceling one checkpoint-backed BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowCancelRequest {
    /// Workflow instance identifier used for checkpoint lookup and deletion.
    pub instance_id: String,
    /// Checkpoint backend to cancel for this bounded workflow instance.
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

    /// Resolves the checkpoint backend for one bounded workflow run.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when runtime-config lookup
    /// fails or when a local checkpoint path cannot be rooted against the
    /// current working directory.
    pub fn resolve_checkpoint_store(
        &self,
        backend: Option<&QianjiBpmnWorkflowCheckpointBackend>,
    ) -> Result<Option<QianjiBpmnCheckpointStore>, QianjiBpmnWorkflowControlError> {
        service::resolve_checkpoint_store(self, backend)
    }

    /// Resolves paths, loads the BPMN package, resolves checkpoint storage, and
    /// shapes the engine-facing execution request for one bounded workflow run.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when path resolution,
    /// runtime-config lookup, or BPMN/DMN package loading fails.
    pub fn prepare_start_workflow(
        &self,
        request: &QianjiBpmnWorkflowStartRequest,
    ) -> Result<QianjiBpmnPreparedWorkflowStart, QianjiBpmnWorkflowControlError> {
        service::prepare_start_workflow(self, request)
    }

    /// Runs one already-prepared BPMN workflow through the lower-level
    /// execution facade.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the execution facade
    /// cannot create, resume, or advance the workflow instance.
    pub async fn start_prepared_workflow<H: BpmnHostBridge>(
        &self,
        prepared: QianjiBpmnPreparedWorkflowStart,
        host: &H,
    ) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowControlError> {
        service::start_prepared_workflow(self, prepared, host).await
    }

    /// Runs one already-prepared BPMN workflow while reporting newly produced
    /// trace events after each runtime step.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the execution facade
    /// cannot create, resume, or advance the workflow instance.
    pub async fn start_prepared_workflow_with_trace_observer<H, F>(
        &self,
        prepared: QianjiBpmnPreparedWorkflowStart,
        host: &H,
        trace_observer: F,
    ) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowControlError>
    where
        H: BpmnHostBridge,
        F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
    {
        service::start_prepared_workflow_with_trace_observer(self, prepared, host, trace_observer)
            .await
    }

    /// Runs one already-prepared BPMN workflow until the next host boundary or
    /// another stable outcome while reporting newly produced trace events.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the execution facade
    /// cannot create, resume, or advance the workflow instance.
    pub async fn start_prepared_workflow_until_host_boundary<H, F>(
        &self,
        prepared: QianjiBpmnPreparedWorkflowStart,
        host: &H,
        resolve_initial_host_work: bool,
        trace_observer: F,
    ) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowControlError>
    where
        H: BpmnHostBridge,
        F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
    {
        service::start_prepared_workflow_until_host_boundary(
            self,
            prepared,
            host,
            resolve_initial_host_work,
            trace_observer,
        )
        .await
    }

    /// Prepares and runs one bounded BPMN workflow in a single step.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when path resolution,
    /// package loading, checkpoint backend resolution, or execution fails.
    pub async fn start_workflow<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnWorkflowStartRequest,
        host: &H,
    ) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowControlError> {
        service::start_workflow(self, request, host).await
    }

    /// Resolves paths, loads the checkpoint-backed workflow identity, loads
    /// the BPMN package, resolves checkpoint storage, and shapes the
    /// engine-facing execution request for one bounded workflow resume.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when path resolution,
    /// runtime-config lookup, checkpoint loading, or BPMN/DMN package loading
    /// fails.
    pub async fn prepare_resume_workflow(
        &self,
        request: &QianjiBpmnWorkflowResumeRequest,
    ) -> Result<QianjiBpmnPreparedWorkflowResume, QianjiBpmnWorkflowControlError> {
        service::prepare_resume_workflow(self, request).await
    }

    /// Runs one already-prepared checkpoint-backed BPMN workflow through the
    /// lower-level execution facade.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the execution facade
    /// cannot resume or advance the workflow instance.
    pub async fn resume_prepared_workflow<H: BpmnHostBridge>(
        &self,
        prepared: QianjiBpmnPreparedWorkflowResume,
        host: &H,
    ) -> Result<QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowControlError> {
        service::resume_prepared_workflow(self, prepared, host).await
    }

    /// Runs one already-prepared checkpoint-backed BPMN workflow until the next
    /// host boundary or another stable outcome while reporting trace events.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the execution facade
    /// cannot resume or advance the workflow instance.
    pub async fn resume_prepared_workflow_until_host_boundary<H, F>(
        &self,
        prepared: QianjiBpmnPreparedWorkflowResume,
        host: &H,
        resolve_initial_host_work: bool,
        trace_observer: F,
    ) -> Result<QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowControlError>
    where
        H: BpmnHostBridge,
        F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
    {
        service::resume_prepared_workflow_until_host_boundary(
            self,
            prepared,
            host,
            resolve_initial_host_work,
            trace_observer,
        )
        .await
    }

    /// Prepares and resumes one checkpoint-backed BPMN workflow in a single
    /// step.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when path resolution,
    /// package loading, checkpoint lookup, or resumed execution fails.
    pub async fn resume_workflow<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnWorkflowResumeRequest,
        host: &H,
    ) -> Result<QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowControlError> {
        service::resume_workflow(self, request, host).await
    }

    /// Polls external events for one checkpoint-backed BPMN workflow through
    /// the same checkpoint continuation path used by generic resume.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when path resolution,
    /// package loading, checkpoint lookup, or event-poll execution fails.
    pub async fn poll_workflow_events<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnWorkflowEventPollRequest,
        host: &H,
    ) -> Result<QianjiBpmnWorkflowEventPollReport, QianjiBpmnWorkflowControlError> {
        service::poll_workflow_events(self, request, host).await
    }

    /// Completes pending host work for one checkpoint-backed BPMN workflow
    /// through the same checkpoint continuation path used by generic resume.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when path resolution,
    /// package loading, checkpoint lookup, or host-task completion fails.
    pub async fn complete_workflow_task<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnWorkflowTaskCompleteRequest,
        host: &H,
    ) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowControlError> {
        service::complete_workflow_task(self, request, host).await
    }

    /// Loads one checkpoint-backed BPMN workflow status without re-running the
    /// execution driver.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when runtime-config lookup or
    /// checkpoint loading fails, or when the requested checkpoint does not
    /// exist.
    pub async fn load_workflow_status(
        &self,
        request: &QianjiBpmnWorkflowStatusRequest,
    ) -> Result<QianjiBpmnWorkflowStatusReport, QianjiBpmnWorkflowControlError> {
        service::load_workflow_status(self, request).await
    }

    /// Cancels one checkpoint-backed BPMN workflow without re-running the
    /// execution driver.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when runtime-config lookup,
    /// checkpoint loading, or checkpoint deletion fails, or when the requested
    /// checkpoint does not exist.
    pub async fn cancel_workflow(
        &self,
        request: &QianjiBpmnWorkflowCancelRequest,
    ) -> Result<QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowControlError> {
        service::cancel_workflow(self, request).await
    }
}
