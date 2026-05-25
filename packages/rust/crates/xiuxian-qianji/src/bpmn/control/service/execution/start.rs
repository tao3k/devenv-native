use crate::bpmn::control::{
    QianjiBpmnPreparedWorkflowStart, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowStartReport,
    QianjiBpmnWorkflowStartRequest,
};
use crate::bpmn::control_service::checkpoint::resolve_checkpoint_store;
use crate::bpmn::control_service::pathing::resolve_path_against_current_dir;
use crate::bpmn::driver::QianjiBpmnExecutionRequest;
use crate::bpmn::execution::QianjiBpmnExecutionFacade;
use crate::bpmn::loader::load_bpmn_package_from_files;
use crate::bpmn::session::QianjiBpmnSession;
use crate::telemetry::unix_millis_now;
use xiuxian_qianji_bpmn_engine::{BpmnExecutionTraceEvent, BpmnHostBridge};

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
            request.process_id.clone(),
            request.instance_id.clone(),
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
