//! Bounded Flowhub service worker helpers for qianji-server.
//!
//! This module keeps the server integration reusable without adding another
//! command surface. The control ledger remains the durable activity authority;
//! the BPMN checkpoint service remains the workflow frontier authority.

use std::io;
use std::path::Path;

use serde_json::Value;
use xiuxian_qianji_bpmn_engine::{BpmnHostBridge, PendingHostWork, PendingHostWorkKind};
use xiuxian_qianji_control::{
    ActivityJournalWriteOutcome, ControlLedger, HotStateLeasedActivityTask, HotStateStore, RunId,
    TaskQueue, WorkerActivityCompletedRecord, WorkerActivityHotStateMirrorRequest,
    WorkerActivityStartRecord, WorkerActivityTask, WorkerId, WorkerRef,
    mirror_worker_activity_tasks_to_hot_state, record_admitted_activity_task_schedule_idempotent,
    record_worker_activity_completed_idempotent, record_worker_activity_started_idempotent,
};
use xiuxian_qianji_runtime::{
    QianjiRuntimeBpmnInstanceId, QianjiRuntimeBpmnSourcePath,
    QianjiRuntimeContinueUntilHumanBoundary, QianjiRuntimeDmnSourcePaths,
    QianjiRuntimeWorkflowControlPort, QianjiRuntimeWorkflowResumeRequest,
    QianjiRuntimeWorkflowStatusRequest, QianjiRuntimeWorkflowTaskCompleteRequest,
    QianjiRuntimeWorkflowTaskCompletionKind, QianjiRuntimeWorkflowTaskCompletionPayload,
    build_flowhub_service_task_completion,
};

use crate::bpmn::{
    FLOWHUB_SERVICE_COMPLETION_METADATA_KEY, FlowhubScenarioIdRef,
    FlowhubServiceActivityScheduleInput, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowHttpState,
    QianjiBpmnWorkflowTaskCompleteReport, QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs,
    build_flowhub_service_activity_schedule_record,
    build_flowhub_service_task_contract_activity_result,
};

#[derive(Clone)]
/// Request for one bounded qianji-server Flowhub service worker loop.
pub struct QianjiServerFlowhubServiceWorkerLoopRequest<'a> {
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
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    /// Timestamp used for schedule, mirror, and start events.
    pub now_ms: u64,
    /// Hot-state activity lease TTL in milliseconds.
    pub lease_ttl_ms: u64,
    /// Timestamp used for durable activity completion events.
    pub settled_at_ms: u64,
    /// Maximum service tasks to complete before returning.
    pub max_steps: usize,
}

/// Result of one bounded qianji-server Flowhub service worker loop.
#[derive(Debug, Clone)]
pub struct QianjiServerFlowhubServiceWorkerLoopOutput {
    /// Completed service-task steps in deterministic execution order.
    pub completed_steps: Vec<QianjiServerFlowhubServiceWorkerStepOutput>,
    /// Remaining BPMN pending host-work count after the loop.
    pub final_pending_host_work_count: usize,
    /// Last BPMN task-completion report, if any service task was completed.
    pub final_report: Option<QianjiBpmnWorkflowTaskCompleteReport>,
}

/// Result of one completed Flowhub BPMN service task.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiServerFlowhubServiceWorkerStepOutput {
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

/// Runs a bounded qianji-server Flowhub service worker loop.
///
/// The loop reads the current BPMN checkpoint frontier, schedules the next
/// Flowhub service task into the control ledger, mirrors it into hot state,
/// claims it, derives deterministic contract completion data, records durable
/// worker lifecycle events, and completes the BPMN service task through the
/// workflow control service.
///
/// # Errors
///
/// Returns an I/O error when workflow checkpoint loading, activity scheduling,
/// hot-state claim/release, durable lifecycle journaling, Flowhub completion
/// derivation, or BPMN task completion fails.
pub async fn run_qianji_server_flowhub_service_worker_completion_loop<L, H, B>(
    state: &QianjiBpmnWorkflowHttpState<B>,
    ledger: &L,
    hot_state: &H,
    request: &QianjiServerFlowhubServiceWorkerLoopRequest<'_>,
) -> io::Result<QianjiServerFlowhubServiceWorkerLoopOutput>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
    B: BpmnHostBridge + Clone + Send + Sync,
{
    let mut completed_steps = Vec::new();
    let mut final_report = None;
    for step_index in 0..request.max_steps {
        let Some(pending_work) = load_next_service_work(state, request).await? else {
            break;
        };
        let step =
            complete_one_service_work(state, ledger, hot_state, request, step_index, &pending_work)
                .await?;
        final_report = step.report;
        completed_steps.push(step.output);
    }
    let final_pending_host_work_count = load_pending_host_work_count(state, request).await?;
    Ok(QianjiServerFlowhubServiceWorkerLoopOutput {
        completed_steps,
        final_pending_host_work_count,
        final_report,
    })
}

struct CompletedServiceWork {
    output: QianjiServerFlowhubServiceWorkerStepOutput,
    report: Option<QianjiBpmnWorkflowTaskCompleteReport>,
}

async fn complete_one_service_work<L, H, B>(
    state: &QianjiBpmnWorkflowHttpState<B>,
    ledger: &L,
    hot_state: &H,
    request: &QianjiServerFlowhubServiceWorkerLoopRequest<'_>,
    step_index: usize,
    pending_work: &PendingHostWork,
) -> io::Result<CompletedServiceWork>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
    B: BpmnHostBridge + Clone + Send + Sync,
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
    let report =
        complete_bpmn_service_work(state, request, &claimed.activity_task.task, completion_data)
            .await?;
    let released = hot_state
        .release_activity_task_lease(&claimed.lease)
        .await
        .map_err(control_io_error)?;
    let output = QianjiServerFlowhubServiceWorkerStepOutput {
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

async fn complete_bpmn_service_work<B>(
    state: &QianjiBpmnWorkflowHttpState<B>,
    request: &QianjiServerFlowhubServiceWorkerLoopRequest<'_>,
    task: &WorkerActivityTask,
    completion_data: Value,
) -> io::Result<QianjiBpmnWorkflowTaskCompleteReport>
where
    B: BpmnHostBridge + Clone + Send + Sync,
{
    let complete_request =
        build_runtime_service_task_complete_request(request, task, completion_data)?;
    let resume_request = QianjiRuntimeWorkflowResumeRequest {
        bpmn_source: QianjiRuntimeBpmnSourcePath::new(request.bpmn_source.to_path_buf()),
        dmn_sources: QianjiRuntimeDmnSourcePaths::empty(),
        instance_id: QianjiRuntimeBpmnInstanceId::new(request.instance_id),
        checkpoint_backend: complete_request.checkpoint_backend.clone(),
    };
    let prepared = <QianjiBpmnWorkflowControlService as QianjiRuntimeWorkflowControlPort<
        B,
    >>::prepare_resume_workflow(&state.service, resume_request)
    .await
    .map_err(control_io_error)?;
    <QianjiBpmnWorkflowControlService as QianjiRuntimeWorkflowControlPort<
        B,
    >>::complete_prepared_workflow_task_until_host_boundary(
        &state.service,
        prepared,
        complete_request,
        &state.host,
    )
    .await
    .map_err(control_io_error)
}

fn build_runtime_service_task_complete_request(
    request: &QianjiServerFlowhubServiceWorkerLoopRequest<'_>,
    task: &WorkerActivityTask,
    completion_data: Value,
) -> io::Result<QianjiRuntimeWorkflowTaskCompleteRequest<QianjiBpmnWorkflowCheckpointBackend>> {
    let completion =
        build_flowhub_service_task_completion(task, completion_data).map_err(control_io_error)?;
    Ok(QianjiRuntimeWorkflowTaskCompleteRequest {
        bpmn_source: QianjiRuntimeBpmnSourcePath::new(request.bpmn_source.to_path_buf()),
        dmn_sources: QianjiRuntimeDmnSourcePaths::empty(),
        instance_id: QianjiRuntimeBpmnInstanceId::new(request.instance_id),
        checkpoint_backend: request.checkpoint_backend.clone(),
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

async fn load_next_service_work<B>(
    state: &QianjiBpmnWorkflowHttpState<B>,
    request: &QianjiServerFlowhubServiceWorkerLoopRequest<'_>,
) -> io::Result<Option<PendingHostWork>>
where
    B: BpmnHostBridge + Clone + Send + Sync,
{
    let status = <QianjiBpmnWorkflowControlService as QianjiRuntimeWorkflowControlPort<
        B,
    >>::load_workflow_status_view(
        &state.service,
        QianjiRuntimeWorkflowStatusRequest {
            instance_id: QianjiRuntimeBpmnInstanceId::new(request.instance_id),
            checkpoint_backend: request.checkpoint_backend.clone(),
        },
    )
        .await
        .map_err(control_io_error)?;
    Ok(status
        .pending_host_work
        .into_iter()
        .find(|work| work.kind == PendingHostWorkKind::Service))
}

async fn load_pending_host_work_count<B>(
    state: &QianjiBpmnWorkflowHttpState<B>,
    request: &QianjiServerFlowhubServiceWorkerLoopRequest<'_>,
) -> io::Result<usize>
where
    B: BpmnHostBridge + Clone + Send + Sync,
{
    let status = <QianjiBpmnWorkflowControlService as QianjiRuntimeWorkflowControlPort<
        B,
    >>::load_workflow_status_view(
        &state.service,
        QianjiRuntimeWorkflowStatusRequest {
            instance_id: QianjiRuntimeBpmnInstanceId::new(request.instance_id),
            checkpoint_backend: request.checkpoint_backend.clone(),
        },
    )
        .await
        .map_err(control_io_error)?;
    Ok(status.pending_host_work.len())
}

async fn claim_flowhub_service_task<H>(
    hot_state: &H,
    request: &QianjiServerFlowhubServiceWorkerLoopRequest<'_>,
    task_queue: &TaskQueue,
    now_ms: u64,
) -> io::Result<HotStateLeasedActivityTask>
where
    H: HotStateStore + ?Sized,
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
