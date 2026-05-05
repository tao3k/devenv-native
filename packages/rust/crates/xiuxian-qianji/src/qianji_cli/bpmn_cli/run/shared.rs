#[cfg(test)]
use crate::qianji_cli::bpmn_cli::deps::{
    QianjiBpmnCheckpointStore, QianjiBpmnWorkflowCheckpointBackend,
};
use crate::qianji_cli::bpmn_cli::deps::{
    QianjiBpmnWorkflowControlService, QianjiRuntimeEnv, SchedulerAgentIdentity,
};

pub(super) fn workflow_control_service(
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> QianjiBpmnWorkflowControlService {
    let mut service = QianjiBpmnWorkflowControlService::new();
    if let Some(runtime_env) = runtime_env.cloned() {
        service = service.with_runtime_env(runtime_env);
    }
    if let Some(scheduler_identity) = scheduler_identity.cloned() {
        service = service.with_scheduler_identity(scheduler_identity);
    }
    service
}

#[cfg(test)]
pub(crate) fn resolve_bpmn_checkpoint_store_with_env(
    backend: Option<&QianjiBpmnWorkflowCheckpointBackend>,
    runtime_env: Option<&QianjiRuntimeEnv>,
) -> Result<Option<QianjiBpmnCheckpointStore>, Box<dyn std::error::Error>> {
    workflow_control_service(runtime_env, None)
        .resolve_checkpoint_store(backend)
        .map_err(Into::into)
}
