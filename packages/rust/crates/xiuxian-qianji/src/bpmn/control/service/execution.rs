use super::checkpoint::{load_required_checkpoint, resolve_checkpoint_store};
use super::pathing::resolve_path_against_current_dir;
use crate::bpmn::control::{
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnPreparedWorkflowStart,
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowEventPollReport, QianjiBpmnWorkflowEventPollRequest,
    QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowStartReport,
    QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowTaskCompleteReport,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload,
};
use crate::bpmn::driver::{QianjiBpmnExecutionRequest, QianjiBpmnPendingHostCompletion};
use crate::bpmn::error::BpmnOrchestrationError;
use crate::bpmn::execution::QianjiBpmnExecutionFacade;
use crate::bpmn::loader::load_bpmn_package_from_files;
use crate::bpmn::session::QianjiBpmnSession;
use crate::telemetry::unix_millis_now;
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnExecutionTraceEvent, BpmnHostBridge, ManualTaskOutcome,
    PendingHostWorkResult, ScriptTaskOutcome, SendTaskOutcome, ServiceTaskOutcome, UserTaskOutcome,
};
use std::io;

pub(crate) fn prepare_start_workflow(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowStartRequest,
) -> Result<QianjiBpmnPreparedWorkflowStart, QianjiBpmnWorkflowControlError> {
    let resolved_bpmn_path = resolve_path_against_current_dir(request.bpmn_path.as_path())?;
    let resolved_dmn_paths = request
        .dmn_paths
        .iter()
        .map(|path| resolve_path_against_current_dir(path.as_path()))
        .collect::<Result<Vec<_>, _>>()?;
    let package = load_bpmn_package_from_files(&resolved_bpmn_path, &resolved_dmn_paths)?;
    let checkpoint_store = resolve_checkpoint_store(service, request.checkpoint_backend.as_ref())?;

    Ok(QianjiBpmnPreparedWorkflowStart {
        package,
        resolved_bpmn_path,
        resolved_dmn_paths,
        checkpoint_store,
        execution_request: QianjiBpmnExecutionRequest::new(
            &request.process_id,
            &request.instance_id,
            request.initial_variables.clone(),
            unix_millis_now(),
        )
        .with_start_at_node_id(request.start_at_node_id.clone()),
        loaded_checkpoint: None,
    })
}

pub(crate) async fn start_prepared_workflow<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowStart,
    host: &H,
) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowControlError> {
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(prepared.package, prepared.checkpoint_store.clone());
    if let Some(scheduler_identity) = service.scheduler_identity.clone() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let execution = execution_facade
        .run(&prepared.execution_request, host)
        .await?;

    Ok(QianjiBpmnWorkflowStartReport {
        resolved_bpmn_path: prepared.resolved_bpmn_path,
        resolved_dmn_paths: prepared.resolved_dmn_paths,
        checkpoint_store: prepared.checkpoint_store,
        execution,
    })
}

pub(crate) async fn start_prepared_workflow_with_trace_observer<H, F>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowStart,
    host: &H,
    trace_observer: F,
) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowControlError>
where
    H: BpmnHostBridge,
    F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
{
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(prepared.package, prepared.checkpoint_store.clone());
    if let Some(scheduler_identity) = service.scheduler_identity.clone() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let execution = execution_facade
        .run_with_trace_observer(&prepared.execution_request, host, trace_observer)
        .await?;

    Ok(QianjiBpmnWorkflowStartReport {
        resolved_bpmn_path: prepared.resolved_bpmn_path,
        resolved_dmn_paths: prepared.resolved_dmn_paths,
        checkpoint_store: prepared.checkpoint_store,
        execution,
    })
}

pub(crate) async fn start_prepared_workflow_until_host_boundary<H, F>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowStart,
    host: &H,
    resolve_initial_host_work: bool,
    trace_observer: F,
) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowControlError>
where
    H: BpmnHostBridge,
    F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
{
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(prepared.package, prepared.checkpoint_store.clone());
    if let Some(scheduler_identity) = service.scheduler_identity.clone() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let execution = execution_facade
        .run_until_host_boundary_with_trace_observer(
            &prepared.execution_request,
            host,
            resolve_initial_host_work,
            trace_observer,
        )
        .await?;

    Ok(QianjiBpmnWorkflowStartReport {
        resolved_bpmn_path: prepared.resolved_bpmn_path,
        resolved_dmn_paths: prepared.resolved_dmn_paths,
        checkpoint_store: prepared.checkpoint_store,
        execution,
    })
}

pub(crate) async fn start_prepared_workflow_until_human_boundary<H, F>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowStart,
    host: &H,
    resolve_initial_host_work: bool,
    trace_observer: F,
) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowControlError>
where
    H: BpmnHostBridge,
    F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
{
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(prepared.package, prepared.checkpoint_store.clone());
    if let Some(scheduler_identity) = service.scheduler_identity.clone() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let execution = execution_facade
        .run_until_human_boundary_with_trace_observer(
            &prepared.execution_request,
            host,
            resolve_initial_host_work,
            trace_observer,
        )
        .await?;

    Ok(QianjiBpmnWorkflowStartReport {
        resolved_bpmn_path: prepared.resolved_bpmn_path,
        resolved_dmn_paths: prepared.resolved_dmn_paths,
        checkpoint_store: prepared.checkpoint_store,
        execution,
    })
}

pub(crate) async fn start_workflow<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowStartRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowControlError> {
    let prepared = prepare_start_workflow(service, request)?;
    start_prepared_workflow(service, prepared, host).await
}

pub(crate) async fn prepare_resume_workflow(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowResumeRequest,
) -> Result<QianjiBpmnPreparedWorkflowResume, QianjiBpmnWorkflowControlError> {
    let resolved_bpmn_path = resolve_path_against_current_dir(request.bpmn_path.as_path())?;
    let resolved_dmn_paths = request
        .dmn_paths
        .iter()
        .map(|path| resolve_path_against_current_dir(path.as_path()))
        .collect::<Result<Vec<_>, _>>()?;
    let (checkpoint_store, checkpoint) =
        load_required_checkpoint(service, &request.instance_id, &request.checkpoint_backend)
            .await?;
    let package = load_bpmn_package_from_files(&resolved_bpmn_path, &resolved_dmn_paths)?;
    let process_id = checkpoint.state.process.process_id.to_string();

    Ok(QianjiBpmnPreparedWorkflowStart {
        package,
        resolved_bpmn_path,
        resolved_dmn_paths,
        checkpoint_store: Some(checkpoint_store),
        execution_request: QianjiBpmnExecutionRequest::new(
            &process_id,
            &request.instance_id,
            None,
            unix_millis_now(),
        ),
        loaded_checkpoint: Some(checkpoint),
    })
}

pub(crate) async fn prepare_resume_workflow_from_prepared_start(
    _service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowResumeRequest,
    prepared_start: &QianjiBpmnPreparedWorkflowStart,
) -> Result<QianjiBpmnPreparedWorkflowResume, QianjiBpmnWorkflowControlError> {
    let checkpoint_store = prepared_start
        .checkpoint_store
        .clone()
        .ok_or_else(|| io::Error::other("prepared workflow resume requires a checkpoint store"))?;
    let checkpoint = checkpoint_store
        .load(request.instance_id.as_str())
        .await?
        .ok_or_else(|| QianjiBpmnWorkflowControlError::CheckpointMissing {
            instance_id: request.instance_id.clone(),
        })?;
    let process_id = checkpoint.state.process.process_id.to_string();

    Ok(QianjiBpmnPreparedWorkflowStart {
        package: prepared_start.package.clone(),
        resolved_bpmn_path: prepared_start.resolved_bpmn_path.clone(),
        resolved_dmn_paths: prepared_start.resolved_dmn_paths.clone(),
        checkpoint_store: Some(checkpoint_store),
        execution_request: QianjiBpmnExecutionRequest::new(
            &process_id,
            &request.instance_id,
            None,
            unix_millis_now(),
        ),
        loaded_checkpoint: Some(checkpoint),
    })
}

pub(crate) async fn resume_prepared_workflow<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowResume,
    host: &H,
) -> Result<QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowControlError> {
    start_prepared_workflow(service, prepared, host).await
}

pub(crate) async fn resume_prepared_workflow_until_host_boundary<H, F>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowResume,
    host: &H,
    resolve_initial_host_work: bool,
    trace_observer: F,
) -> Result<QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowControlError>
where
    H: BpmnHostBridge,
    F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
{
    start_prepared_workflow_until_host_boundary(
        service,
        prepared,
        host,
        resolve_initial_host_work,
        trace_observer,
    )
    .await
}

pub(crate) async fn resume_prepared_workflow_until_human_boundary<H, F>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowResume,
    host: &H,
    resolve_initial_host_work: bool,
    trace_observer: F,
) -> Result<QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowControlError>
where
    H: BpmnHostBridge,
    F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
{
    start_prepared_workflow_until_human_boundary(
        service,
        prepared,
        host,
        resolve_initial_host_work,
        trace_observer,
    )
    .await
}

pub(crate) async fn resume_workflow<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowResumeRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowControlError> {
    let prepared = prepare_resume_workflow(service, request).await?;
    resume_prepared_workflow(service, prepared, host).await
}

pub(crate) async fn poll_workflow_events<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowEventPollRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowEventPollReport, QianjiBpmnWorkflowControlError> {
    let resume_request = QianjiBpmnWorkflowResumeRequest {
        bpmn_path: request.bpmn_path.clone(),
        dmn_paths: request.dmn_paths.clone(),
        instance_id: request.instance_id.clone(),
        checkpoint_backend: request.checkpoint_backend.clone(),
    };
    resume_workflow(service, &resume_request, host).await
}

pub(crate) async fn complete_workflow_task<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowTaskCompleteRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowControlError> {
    let resume_request = QianjiBpmnWorkflowResumeRequest {
        bpmn_path: request.bpmn_path.clone(),
        dmn_paths: request.dmn_paths.clone(),
        instance_id: request.instance_id.clone(),
        checkpoint_backend: request.checkpoint_backend.clone(),
    };
    let prepared = prepare_resume_workflow(service, &resume_request).await?;
    complete_prepared_workflow_task(service, prepared, request, host).await
}

pub(crate) async fn complete_prepared_workflow_task<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowResume,
    request: &QianjiBpmnWorkflowTaskCompleteRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowControlError> {
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(prepared.package, prepared.checkpoint_store.clone());
    if let Some(scheduler_identity) = service.scheduler_identity.clone() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let loaded_checkpoint = prepared.loaded_checkpoint.clone();
    validate_completion_claimant(loaded_checkpoint.as_ref(), &request.completion)?;
    let completion = pending_host_completion_from_completion(&request.completion);
    let execution = match (request.continue_until_human_boundary, loaded_checkpoint) {
        (true, Some(checkpoint)) => {
            execution_facade
                .complete_pending_host_work_from_checkpoint_until_human_boundary(
                    &prepared.execution_request,
                    checkpoint,
                    completion,
                    host,
                )
                .await?
        }
        (false, Some(checkpoint)) => {
            execution_facade
                .complete_pending_host_work_from_checkpoint(
                    &prepared.execution_request,
                    checkpoint,
                    completion,
                    host,
                )
                .await?
        }
        (true, None) => {
            execution_facade
                .complete_pending_host_work_until_human_boundary(
                    &prepared.execution_request,
                    completion,
                    host,
                )
                .await?
        }
        (false, None) => {
            execution_facade
                .complete_pending_host_work(&prepared.execution_request, completion, host)
                .await?
        }
    };

    Ok(QianjiBpmnWorkflowTaskCompleteReport {
        resolved_bpmn_path: prepared.resolved_bpmn_path,
        resolved_dmn_paths: prepared.resolved_dmn_paths,
        checkpoint_store: prepared.checkpoint_store,
        execution,
    })
}

pub(crate) async fn complete_prepared_workflow_task_until_host_boundary<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowResume,
    request: &QianjiBpmnWorkflowTaskCompleteRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowControlError> {
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(prepared.package, prepared.checkpoint_store.clone());
    if let Some(scheduler_identity) = service.scheduler_identity.clone() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let loaded_checkpoint = prepared.loaded_checkpoint.clone();
    validate_completion_claimant(loaded_checkpoint.as_ref(), &request.completion)?;
    let completion = pending_host_completion_from_completion(&request.completion);
    let execution = match loaded_checkpoint {
        Some(checkpoint) => {
            execution_facade
                .complete_pending_host_work_from_checkpoint_until_host_boundary(
                    &prepared.execution_request,
                    checkpoint,
                    completion,
                    host,
                )
                .await?
        }
        None => {
            execution_facade
                .complete_pending_host_work_until_host_boundary(
                    &prepared.execution_request,
                    completion,
                    host,
                )
                .await?
        }
    };

    Ok(QianjiBpmnWorkflowTaskCompleteReport {
        resolved_bpmn_path: prepared.resolved_bpmn_path,
        resolved_dmn_paths: prepared.resolved_dmn_paths,
        checkpoint_store: prepared.checkpoint_store,
        execution,
    })
}

fn validate_completion_claimant(
    checkpoint: Option<&BpmnCheckpointEnvelope>,
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> Result<(), QianjiBpmnWorkflowControlError> {
    let Some(checkpoint) = checkpoint else {
        return Ok(());
    };
    let Some(pending) = checkpoint
        .state
        .pending_host_work
        .iter()
        .find(|work| work.token_id == completion.token_id)
    else {
        return Ok(());
    };
    let Some(claim) = pending.claim.as_ref() else {
        return Ok(());
    };

    let Some(actual_claimant) = completion
        .claimant
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return Err(BpmnOrchestrationError::PendingHostWorkClaimRequired {
            instance_id: checkpoint.state.instance_id.to_string(),
            token_id: completion.token_id,
            claimed_by: claim.claimant.clone(),
        }
        .into());
    };

    if actual_claimant == claim.claimant {
        return Ok(());
    }

    Err(BpmnOrchestrationError::PendingHostWorkClaimantMismatch {
        instance_id: checkpoint.state.instance_id.to_string(),
        token_id: completion.token_id,
        expected_claimant: claim.claimant.clone(),
        actual_claimant: actual_claimant.to_string(),
    }
    .into())
}

fn pending_host_completion_from_completion(
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> QianjiBpmnPendingHostCompletion {
    QianjiBpmnPendingHostCompletion::new(
        completion.token_id,
        completion.process_id.clone(),
        completion.activity_id.clone(),
        pending_host_work_result_from_completion(completion),
    )
}

fn pending_host_work_result_from_completion(
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> PendingHostWorkResult {
    match completion.kind {
        QianjiBpmnWorkflowTaskCompletionKind::Send => {
            PendingHostWorkResult::Send(SendTaskOutcome {
                data: completion.data.clone(),
            })
        }
        QianjiBpmnWorkflowTaskCompletionKind::Service => {
            PendingHostWorkResult::Service(ServiceTaskOutcome {
                data: completion.data.clone(),
            })
        }
        QianjiBpmnWorkflowTaskCompletionKind::Script => {
            PendingHostWorkResult::Script(ScriptTaskOutcome {
                data: completion.data.clone(),
            })
        }
        QianjiBpmnWorkflowTaskCompletionKind::User => {
            PendingHostWorkResult::User(UserTaskOutcome {
                data: completion.data.clone(),
            })
        }
        QianjiBpmnWorkflowTaskCompletionKind::Manual => {
            PendingHostWorkResult::Manual(ManualTaskOutcome {
                data: completion.data.clone(),
            })
        }
    }
}
