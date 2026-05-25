use super::{
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnPreparedWorkflowStart,
    QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowCancelRequest,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowEventPollReport,
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowInstancesReport,
    QianjiBpmnWorkflowInstancesRequest, QianjiBpmnWorkflowInterruptReport,
    QianjiBpmnWorkflowInterruptRequest, QianjiBpmnWorkflowResumeReport,
    QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowStartRequest,
    QianjiBpmnWorkflowStatusReport, QianjiBpmnWorkflowStatusRequest,
    QianjiBpmnWorkflowTaskClaimReport, QianjiBpmnWorkflowTaskClaimRequest,
    QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowTaskCompleteRequest,
    QianjiBpmnWorkflowTaskReleaseReport, QianjiBpmnWorkflowTaskReleaseRequest,
    QianjiBpmnWorkflowWorklistReport, QianjiBpmnWorkflowWorklistRequest,
};
use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::control_service as service;
use crate::bpmn::session::QianjiBpmnSession;
use xiuxian_qianji_bpmn_engine::{BpmnExecutionTraceEvent, BpmnHostBridge};

impl QianjiBpmnWorkflowControlService {
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

    /// Runs one already-prepared BPMN workflow through non-human host work
    /// until the next user/manual boundary or another stable outcome.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the execution facade
    /// cannot create, resume, or advance the workflow instance.
    pub async fn start_prepared_workflow_until_human_boundary<H, F>(
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
        service::start_prepared_workflow_until_human_boundary(
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

    /// Prepares a checkpoint-backed BPMN resume by reusing the package,
    /// resolved source paths, and checkpoint store from an already prepared
    /// start.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the prepared start has
    /// no checkpoint store, the checkpoint cannot be loaded, or the requested
    /// checkpoint does not exist.
    pub async fn prepare_resume_workflow_from_prepared_start(
        &self,
        request: &QianjiBpmnWorkflowResumeRequest,
        prepared_start: &QianjiBpmnPreparedWorkflowStart,
    ) -> Result<QianjiBpmnPreparedWorkflowResume, QianjiBpmnWorkflowControlError> {
        service::prepare_resume_workflow_from_prepared_start(self, request, prepared_start).await
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

    /// Runs one checkpoint-backed BPMN workflow through non-human host work
    /// until the next user/manual boundary or another stable outcome.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the execution facade
    /// cannot resume or advance the workflow instance.
    pub async fn resume_prepared_workflow_until_human_boundary<H, F>(
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
        service::resume_prepared_workflow_until_human_boundary(
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

    /// Completes pending host work against an already prepared checkpoint
    /// resume without reloading the BPMN package.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the execution facade
    /// cannot resume the checkpoint, rejects the explicit completion, or
    /// cannot persist the resulting checkpoint state.
    pub async fn complete_prepared_workflow_task<H: BpmnHostBridge>(
        &self,
        prepared: QianjiBpmnPreparedWorkflowResume,
        request: &QianjiBpmnWorkflowTaskCompleteRequest,
        host: &H,
    ) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowControlError> {
        service::complete_prepared_workflow_task(self, prepared, request, host).await
    }

    /// Completes pending host work against an already prepared checkpoint
    /// resume, then stops at the next host boundary.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when the execution facade
    /// cannot resume the checkpoint, rejects the explicit completion, or
    /// cannot persist the resulting checkpoint state.
    pub async fn complete_prepared_workflow_task_until_host_boundary<H: BpmnHostBridge>(
        &self,
        prepared: QianjiBpmnPreparedWorkflowResume,
        request: &QianjiBpmnWorkflowTaskCompleteRequest,
        host: &H,
    ) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowControlError> {
        service::complete_prepared_workflow_task_until_host_boundary(self, prepared, request, host)
            .await
    }

    /// Claims one checkpoint-backed pending BPMN `userTask` or `manualTask`
    /// without re-running the execution driver.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when runtime-config lookup,
    /// checkpoint loading, claim validation, or checkpoint persistence fails,
    /// or when the requested checkpoint does not exist.
    pub async fn claim_workflow_task(
        &self,
        request: &QianjiBpmnWorkflowTaskClaimRequest,
    ) -> Result<QianjiBpmnWorkflowTaskClaimReport, QianjiBpmnWorkflowControlError> {
        service::claim_workflow_task(self, request).await
    }

    /// Releases one checkpoint-backed pending BPMN `userTask` or `manualTask`
    /// claim without re-running the execution driver.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when runtime-config lookup,
    /// checkpoint loading, release validation, or checkpoint persistence fails,
    /// or when the requested checkpoint does not exist.
    pub async fn release_workflow_task(
        &self,
        request: &QianjiBpmnWorkflowTaskReleaseRequest,
    ) -> Result<QianjiBpmnWorkflowTaskReleaseReport, QianjiBpmnWorkflowControlError> {
        service::release_workflow_task(self, request).await
    }

    /// Lists checkpoint-backed pending human work without re-running the
    /// execution driver.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when runtime-config lookup or
    /// checkpoint enumeration fails.
    pub async fn list_workflow_worklist(
        &self,
        request: &QianjiBpmnWorkflowWorklistRequest,
    ) -> Result<QianjiBpmnWorkflowWorklistReport, QianjiBpmnWorkflowControlError> {
        service::list_workflow_worklist(self, request).await
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

    /// Lists checkpoint-backed BPMN workflow instances without re-running the
    /// execution driver.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when runtime-config lookup or
    /// checkpoint enumeration fails.
    pub async fn list_workflow_instances(
        &self,
        request: &QianjiBpmnWorkflowInstancesRequest,
    ) -> Result<QianjiBpmnWorkflowInstancesReport, QianjiBpmnWorkflowControlError> {
        service::list_workflow_instances(self, request).await
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

    /// Interrupts one checkpoint-backed BPMN workflow without deleting its
    /// durable state.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnWorkflowControlError`] when runtime-config lookup,
    /// checkpoint loading, or checkpoint persistence fails, or when the
    /// requested checkpoint does not exist.
    pub async fn interrupt_workflow(
        &self,
        request: &QianjiBpmnWorkflowInterruptRequest,
    ) -> Result<QianjiBpmnWorkflowInterruptReport, QianjiBpmnWorkflowControlError> {
        service::interrupt_workflow(self, request).await
    }
}
