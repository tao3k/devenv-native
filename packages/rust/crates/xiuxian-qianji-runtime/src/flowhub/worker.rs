//! Runtime-owned bounded Flowhub service worker loop.
//!
//! The loop is generic over the workflow-control port and host bridge. Server
//! crates provide concrete state. This module owns the BPMN/Flowhub execution
//! sequence and composes workflow-neutral qianji-control ledger and hot-state
//! helpers; qianji-control remains the durable history and queue authority.

use std::io;
use std::path::Path;

use serde_json::Value;
use xiuxian_qianji_bpmn_engine::{BpmnHostBridge, PendingHostWork, PendingHostWorkKind};
use xiuxian_qianji_control::{
    ActivityJournalWriteOutcome, ControlLedger, HotStateLeasedActivityTask, HotStateStore,
    RunCreatedJournalRecord, RunId, TaskQueue, WorkerActivityCompletedRecord,
    WorkerActivityHotStateMirrorRequest, WorkerActivityStartRecord, WorkerActivityTask, WorkerId,
    WorkerRef, mirror_worker_activity_tasks_to_hot_state,
    record_admitted_activity_task_schedule_idempotent, record_run_created,
    record_worker_activity_completed_idempotent, record_worker_activity_started_idempotent,
};

use crate::{
    FlowhubScenarioIdRef, FlowhubServiceActivityScheduleInput, QianjiRuntimeBpmnInstanceId,
    QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeBpmnSourcePath,
    QianjiRuntimeContinueUntilHumanBoundary, QianjiRuntimeDmnSourcePaths, QianjiRuntimeInstantMs,
    QianjiRuntimeWorkflowControlPort, QianjiRuntimeWorkflowResumeRequest,
    QianjiRuntimeWorkflowStatusRequest, QianjiRuntimeWorkflowTaskCompleteRequest,
    QianjiRuntimeWorkflowTaskCompletionKind, QianjiRuntimeWorkflowTaskCompletionPayload,
    build_flowhub_service_activity_schedule_record, build_flowhub_service_task_completion,
    build_flowhub_service_task_contract_activity_result,
};

use super::FLOWHUB_SERVICE_COMPLETION_METADATA_KEY;

/// Metadata schema for Flowhub service worker control run creation.
pub const FLOWHUB_SERVICE_WORKER_RUN_SCHEMA: &str = "xiuxian_qianji.flowhub.worker_run.v1";

/// Request for one bounded Flowhub service worker loop.
#[derive(Clone)]
pub struct FlowhubServiceWorkerLoopRequest<'a, C> {
    /// Control-plane run that owns the durable activity task journal.
    pub run_id: &'a RunId,
    /// Flowhub scenario id used to derive the service task queue.
    pub scenario_id: &'a str,
    /// BPMN workflow instance id stored in the checkpoint backend.
    pub instance_id: &'a str,
    /// BPMN source used by the checkpoint-backed workflow.
    pub bpmn_source: &'a Path,
    /// Worker identity recorded on durable activity start events.
    pub worker_id: &'a str,
    /// Checkpoint backend that owns the BPMN workflow frontier.
    pub checkpoint_backend: C,
    /// Timestamp used for schedule, mirror, and start events.
    pub now_ms: u64,
    /// Hot-state activity lease TTL in milliseconds.
    pub lease_ttl_ms: u64,
    /// Timestamp used for durable activity completion events.
    pub settled_at_ms: u64,
    /// Maximum service tasks to complete before returning.
    pub max_steps: usize,
}

impl<C: Clone> FlowhubServiceWorkerLoopRequest<'_, C> {
    /// Builds a runtime workflow status request for the configured frontier.
    #[must_use]
    pub fn workflow_status_request(&self) -> QianjiRuntimeWorkflowStatusRequest<C> {
        QianjiRuntimeWorkflowStatusRequest {
            instance_id: QianjiRuntimeBpmnInstanceId::new(self.instance_id),
            checkpoint_backend: self.checkpoint_backend.clone(),
        }
    }

    /// Builds a runtime workflow resume request for the configured frontier.
    #[must_use]
    pub fn workflow_resume_request(&self) -> QianjiRuntimeWorkflowResumeRequest<C> {
        QianjiRuntimeWorkflowResumeRequest {
            bpmn_source: QianjiRuntimeBpmnSourcePath::new(self.bpmn_source.to_path_buf()),
            dmn_sources: QianjiRuntimeDmnSourcePaths::empty(),
            instance_id: QianjiRuntimeBpmnInstanceId::new(self.instance_id),
            checkpoint_backend: self.checkpoint_backend.clone(),
        }
    }

    /// Builds a runtime workflow completion request for one Flowhub service task.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the worker activity task does not satisfy the
    /// Flowhub service-task completion contract.
    pub fn workflow_service_task_complete_request(
        &self,
        task: &WorkerActivityTask,
        completion_data: Value,
    ) -> io::Result<QianjiRuntimeWorkflowTaskCompleteRequest<C>> {
        let completion = build_flowhub_service_task_completion(task, completion_data)
            .map_err(control_io_error)?;
        Ok(QianjiRuntimeWorkflowTaskCompleteRequest {
            bpmn_source: QianjiRuntimeBpmnSourcePath::new(self.bpmn_source.to_path_buf()),
            dmn_sources: QianjiRuntimeDmnSourcePaths::empty(),
            instance_id: QianjiRuntimeBpmnInstanceId::new(self.instance_id),
            checkpoint_backend: self.checkpoint_backend.clone(),
            completion: QianjiRuntimeWorkflowTaskCompletionPayload {
                token_id: completion.token_id,
                process_id: completion.process_id,
                activity_id: completion.activity_id,
                kind: QianjiRuntimeWorkflowTaskCompletionKind::Service,
                data: completion.data,
                claimant: completion.claimant,
            },
            continue_until_human_boundary: QianjiRuntimeContinueUntilHumanBoundary::new(false),
        })
    }
}

/// Result of one bounded Flowhub service worker loop.
#[derive(Debug, Clone)]
pub struct FlowhubServiceWorkerLoopOutput<R> {
    /// Completed service-task steps in deterministic execution order.
    pub completed_steps: Vec<FlowhubServiceWorkerStepOutput>,
    /// Remaining BPMN pending host-work count after the loop.
    pub final_pending_host_work_count: usize,
    /// Last BPMN task-completion report, if any service task was completed.
    pub final_report: Option<R>,
}

/// Result of one completed Flowhub BPMN service task.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowhubServiceWorkerStepOutput {
    /// BPMN service task id.
    pub activity_id: String,
    /// BPMN token id completed by this worker step.
    pub token_id: u64,
    /// Number of replay-derived activity tasks mirrored before claim.
    pub mirrored_count: usize,
    /// Durable activity-start journal write outcome.
    pub durable_start: ActivityJournalWriteOutcome,
    /// Durable activity-completion journal write outcome.
    pub durable_terminal: ActivityJournalWriteOutcome,
    /// Whether the hot-state activity lease was released after BPMN completion.
    pub released: bool,
}

/// Runs a bounded Flowhub service worker loop.
///
/// # Errors
///
/// Returns an I/O error when workflow checkpoint loading, activity scheduling,
/// hot-state claim/release, durable lifecycle journaling, Flowhub completion
/// derivation, or BPMN task completion fails.
pub async fn run_flowhub_service_worker_completion_loop<P, L, S, Host>(
    control_port: &P,
    host: &Host,
    ledger: &L,
    hot_state: &S,
    request: &FlowhubServiceWorkerLoopRequest<'_, P::CheckpointBackend>,
) -> io::Result<FlowhubServiceWorkerLoopOutput<P::TaskCompleteReport>>
where
    P: QianjiRuntimeWorkflowControlPort<Host> + ?Sized,
    L: ControlLedger + ?Sized,
    S: HotStateStore + ?Sized,
    Host: BpmnHostBridge + Send + Sync,
{
    ensure_flowhub_service_worker_control_run(ledger, request)?;
    let mut completed_steps = Vec::new();
    let mut final_report = None;
    for step_index in 0..request.max_steps {
        let Some(pending_work) = load_next_service_work(control_port, request).await? else {
            break;
        };
        let step = complete_one_service_work(
            control_port,
            host,
            ledger,
            hot_state,
            request,
            step_index,
            &pending_work,
        )
        .await?;
        final_report = step.report;
        completed_steps.push(step.output);
    }
    let final_pending_host_work_count = load_pending_host_work_count(control_port, request).await?;
    Ok(FlowhubServiceWorkerLoopOutput {
        completed_steps,
        final_pending_host_work_count,
        final_report,
    })
}

fn ensure_flowhub_service_worker_control_run<L, C>(
    ledger: &L,
    request: &FlowhubServiceWorkerLoopRequest<'_, C>,
) -> io::Result<()>
where
    L: ControlLedger + ?Sized,
{
    if !ledger
        .load_events(request.run_id)
        .map_err(control_io_error)?
        .is_empty()
    {
        return Ok(());
    }
    let record = RunCreatedJournalRecord::new(
        request.run_id.clone(),
        format!(
            "Flowhub service worker for scenario {} instance {}",
            request.scenario_id, request.instance_id
        ),
        request.now_ms,
    )
    .with_metadata(serde_json::json!({
        "schema": FLOWHUB_SERVICE_WORKER_RUN_SCHEMA,
        "scenarioId": request.scenario_id,
        "instanceId": request.instance_id,
        "bpmnSource": request.bpmn_source.display().to_string(),
        "workerId": request.worker_id,
        "maxSteps": request.max_steps,
    }));
    record_run_created(ledger, record)
        .map(|_| ())
        .map_err(control_io_error)
}

struct CompletedServiceWork<R> {
    output: FlowhubServiceWorkerStepOutput,
    report: Option<R>,
}

async fn complete_one_service_work<P, L, S, Host>(
    control_port: &P,
    host: &Host,
    ledger: &L,
    hot_state: &S,
    request: &FlowhubServiceWorkerLoopRequest<'_, P::CheckpointBackend>,
    step_index: usize,
    pending_work: &PendingHostWork,
) -> io::Result<CompletedServiceWork<P::TaskCompleteReport>>
where
    P: QianjiRuntimeWorkflowControlPort<Host> + ?Sized,
    L: ControlLedger + ?Sized,
    S: HotStateStore + ?Sized,
    Host: BpmnHostBridge + Send + Sync,
{
    let occurred_at_ms = add_step_offset(request.now_ms, step_index, "now_ms")?;
    let settled_at_ms = add_step_offset(request.settled_at_ms, step_index, "settled_at_ms")?;
    let schedule_record =
        build_flowhub_service_activity_schedule_record(FlowhubServiceActivityScheduleInput {
            run_id: request.run_id,
            occurred_at_ms: QianjiRuntimeInstantMs::from_millis(occurred_at_ms),
            scenario_id: FlowhubScenarioIdRef::new(request.scenario_id),
            instance_id: QianjiRuntimeBpmnInstanceIdRef::new(request.instance_id),
            bpmn_source: request.bpmn_source,
            pending_work,
        })
        .map_err(control_io_error)?;
    let task_queue = schedule_record.task.task_queue.clone();
    record_admitted_activity_task_schedule_idempotent(ledger, schedule_record)
        .map_err(control_io_error)?;
    let mirrored = mirror_worker_activity_tasks_to_hot_state(
        ledger,
        hot_state,
        WorkerActivityHotStateMirrorRequest::new(request.run_id.clone())
            .with_task_queue(task_queue.clone())
            .with_not_before_ms(occurred_at_ms),
    )
    .await
    .map_err(control_io_error)?;
    let claimed =
        claim_flowhub_service_task(hot_state, request, &task_queue, occurred_at_ms).await?;
    let worker_id = WorkerId::new(request.worker_id).map_err(control_io_error)?;
    let durable_start = record_worker_activity_started_idempotent(
        ledger,
        WorkerActivityStartRecord::new(
            claimed.activity_task.task.clone(),
            worker_id,
            occurred_at_ms,
        ),
    )
    .map_err(control_io_error)?;
    let result = build_flowhub_service_task_contract_activity_result(&claimed.activity_task.task)
        .map_err(control_io_error)?;
    let completion_data = flowhub_service_completion_data(&result)?;
    let durable_terminal = record_worker_activity_completed_idempotent(
        ledger,
        WorkerActivityCompletedRecord::new(
            claimed.activity_task.task.clone(),
            settled_at_ms,
            result,
        ),
    )
    .map_err(control_io_error)?;
    let report = complete_bpmn_service_work(
        control_port,
        host,
        request,
        &claimed.activity_task.task,
        completion_data,
    )
    .await?;
    let released = hot_state
        .release_activity_task_lease(&claimed.lease)
        .await
        .map_err(control_io_error)?;
    let output = FlowhubServiceWorkerStepOutput {
        activity_id: pending_activity_id(pending_work)?,
        token_id: pending_work.token_id,
        mirrored_count: mirrored.mirrored_count,
        durable_start,
        durable_terminal,
        released,
    };
    Ok(CompletedServiceWork {
        output,
        report: Some(report),
    })
}

async fn complete_bpmn_service_work<P, Host>(
    control_port: &P,
    host: &Host,
    request: &FlowhubServiceWorkerLoopRequest<'_, P::CheckpointBackend>,
    task: &WorkerActivityTask,
    completion_data: Value,
) -> io::Result<P::TaskCompleteReport>
where
    P: QianjiRuntimeWorkflowControlPort<Host> + ?Sized,
    Host: BpmnHostBridge + Send + Sync,
{
    let complete_request = request.workflow_service_task_complete_request(task, completion_data)?;
    let resume_request = complete_request.workflow_resume_request();
    let prepared = control_port
        .prepare_resume_workflow(resume_request)
        .await
        .map_err(control_io_error)?;
    control_port
        .complete_prepared_workflow_task_until_host_boundary(prepared, complete_request, host)
        .await
        .map_err(control_io_error)
}

async fn load_next_service_work<P, Host>(
    control_port: &P,
    request: &FlowhubServiceWorkerLoopRequest<'_, P::CheckpointBackend>,
) -> io::Result<Option<PendingHostWork>>
where
    P: QianjiRuntimeWorkflowControlPort<Host> + ?Sized,
    Host: BpmnHostBridge + Send + Sync,
{
    let status = control_port
        .load_workflow_status_view(request.workflow_status_request())
        .await
        .map_err(control_io_error)?;
    Ok(status.into_first_pending_host_work_by_kind(&PendingHostWorkKind::Service))
}

async fn load_pending_host_work_count<P, Host>(
    control_port: &P,
    request: &FlowhubServiceWorkerLoopRequest<'_, P::CheckpointBackend>,
) -> io::Result<usize>
where
    P: QianjiRuntimeWorkflowControlPort<Host> + ?Sized,
    Host: BpmnHostBridge + Send + Sync,
{
    let status = control_port
        .load_workflow_status_view(request.workflow_status_request())
        .await
        .map_err(control_io_error)?;
    Ok(status.pending_host_work_count())
}

async fn claim_flowhub_service_task<S, C>(
    hot_state: &S,
    request: &FlowhubServiceWorkerLoopRequest<'_, C>,
    task_queue: &TaskQueue,
    now_ms: u64,
) -> io::Result<HotStateLeasedActivityTask>
where
    S: HotStateStore + ?Sized,
{
    let worker_id = WorkerId::new(request.worker_id).map_err(control_io_error)?;
    hot_state
        .claim_activity_task(
            WorkerRef {
                worker_id,
                capabilities: Vec::new(),
                metadata: Value::Null,
            },
            Some(task_queue),
            now_ms,
            request.lease_ttl_ms,
        )
        .await
        .map_err(control_io_error)?
        .ok_or_else(|| io::Error::other("Flowhub service worker could not claim a mirrored task"))
}

fn flowhub_service_completion_data(
    result: &xiuxian_qianji_control::ActivityResult,
) -> io::Result<Value> {
    result
        .metadata
        .get(FLOWHUB_SERVICE_COMPLETION_METADATA_KEY)
        .and_then(|completion| completion.get("data"))
        .cloned()
        .ok_or_else(|| {
            io::Error::other(format!(
                "Flowhub service executor result is missing `{FLOWHUB_SERVICE_COMPLETION_METADATA_KEY}.data`"
            ))
        })
}

fn pending_activity_id(work: &PendingHostWork) -> io::Result<String> {
    work.activity_id
        .as_ref()
        .map(|activity_id| activity_id.as_str().to_owned())
        .ok_or_else(|| io::Error::other("Flowhub service work is missing activity_id"))
}

fn add_step_offset(base: u64, step_index: usize, field: &'static str) -> io::Result<u64> {
    let offset = u64::try_from(step_index)
        .map_err(|error| io::Error::other(format!("invalid step index for {field}: {error}")))?;
    base.checked_add(offset)
        .ok_or_else(|| io::Error::other(format!("Flowhub service worker {field} overflowed u64")))
}

fn control_io_error(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}
