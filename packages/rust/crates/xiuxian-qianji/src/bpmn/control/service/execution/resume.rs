use super::start::{
    start_prepared_workflow, start_prepared_workflow_until_host_boundary,
    start_prepared_workflow_until_human_boundary,
};
use crate::bpmn::control::{
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnPreparedWorkflowStart,
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowEventPollReport, QianjiBpmnWorkflowEventPollRequest,
    QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowResumeRequest,
};
use crate::bpmn::control_service::checkpoint::load_required_checkpoint;
use crate::bpmn::control_service::pathing::resolve_path_against_current_dir;
use crate::bpmn::driver::QianjiBpmnExecutionRequest;
use crate::bpmn::loader::load_bpmn_package_from_files;
use crate::bpmn::session::QianjiBpmnSession;
use crate::telemetry::unix_millis_now;
use qianji_bpmn_engine::{BpmnExecutionTraceEvent, BpmnHostBridge};
use std::io;

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
            process_id,
            request.instance_id.clone(),
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
            instance_id: request.instance_id.clone().into(),
        })?;
    let process_id = checkpoint.state.process.process_id.to_string();

    Ok(QianjiBpmnPreparedWorkflowStart {
        package: prepared_start.package.clone(),
        resolved_bpmn_path: prepared_start.resolved_bpmn_path.clone(),
        resolved_dmn_paths: prepared_start.resolved_dmn_paths.clone(),
        checkpoint_store: Some(checkpoint_store),
        execution_request: QianjiBpmnExecutionRequest::new(
            process_id,
            request.instance_id.clone(),
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
