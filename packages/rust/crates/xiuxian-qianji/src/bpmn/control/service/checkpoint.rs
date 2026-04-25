use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::control::{
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowInstanceSummary,
    QianjiBpmnWorkflowInstancesReport, QianjiBpmnWorkflowInstancesRequest,
    QianjiBpmnWorkflowStatusReport, QianjiBpmnWorkflowStatusRequest,
};
use crate::runtime_config::{
    resolve_qianji_runtime_checkpoint_config, resolve_qianji_runtime_checkpoint_config_with_env,
};
#[cfg(feature = "duckdb")]
use crate::runtime_config::{
    resolve_qianji_runtime_workflow_state_config,
    resolve_qianji_runtime_workflow_state_config_with_env,
};
use qianji_bpmn_engine::BpmnCheckpointEnvelope;
use std::io;

pub(crate) fn resolve_checkpoint_store(
    service: &QianjiBpmnWorkflowControlService,
    backend: Option<&QianjiBpmnWorkflowCheckpointBackend>,
) -> Result<Option<QianjiBpmnCheckpointStore>, QianjiBpmnWorkflowControlError> {
    match backend {
        None => Ok(None),
        Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey) => {
            let runtime = match service.runtime_env.as_ref() {
                Some(runtime_env) => resolve_qianji_runtime_checkpoint_config_with_env(runtime_env),
                None => resolve_qianji_runtime_checkpoint_config(),
            }
            .map_err(|error| {
                io::Error::other(format!(
                    "failed to resolve Qianji checkpoint runtime config for BPMN workflow control: {error}"
                ))
            })?;
            Ok(Some(
                QianjiBpmnCheckpointStore::from_runtime_checkpoint_config(&runtime),
            ))
        }
        #[cfg(feature = "duckdb")]
        Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb) => {
            let runtime = match service.runtime_env.as_ref() {
                Some(runtime_env) => resolve_qianji_runtime_workflow_state_config_with_env(runtime_env),
                None => resolve_qianji_runtime_workflow_state_config(),
            }
            .map_err(|error| {
                io::Error::other(format!(
                    "failed to resolve Qianji workflow-state runtime config for BPMN workflow control: {error}"
                ))
            })?;
            Ok(Some(QianjiBpmnCheckpointStore::duckdb(
                runtime.local_duckdb_path,
            )))
        }
    }
}

pub(crate) async fn load_workflow_status(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowStatusRequest,
) -> Result<QianjiBpmnWorkflowStatusReport, QianjiBpmnWorkflowControlError> {
    let (checkpoint_store, checkpoint) =
        load_required_checkpoint(service, &request.instance_id, &request.checkpoint_backend)
            .await?;

    Ok(QianjiBpmnWorkflowStatusReport {
        checkpoint_store,
        checkpoint_sequence: checkpoint.sequence,
        instance: checkpoint.state,
    })
}

pub(crate) async fn list_workflow_instances(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowInstancesRequest,
) -> Result<QianjiBpmnWorkflowInstancesReport, QianjiBpmnWorkflowControlError> {
    let checkpoint_store = resolve_checkpoint_store(service, Some(&request.checkpoint_backend))?
        .ok_or_else(|| io::Error::other("workflow operation requires one checkpoint backend"))?;
    let instances = checkpoint_store
        .list()?
        .into_iter()
        .map(QianjiBpmnWorkflowInstanceSummary::from_checkpoint)
        .collect();

    Ok(QianjiBpmnWorkflowInstancesReport {
        checkpoint_store,
        instances,
    })
}

pub(super) async fn load_required_checkpoint(
    service: &QianjiBpmnWorkflowControlService,
    instance_id: &str,
    checkpoint_backend: &QianjiBpmnWorkflowCheckpointBackend,
) -> Result<(QianjiBpmnCheckpointStore, BpmnCheckpointEnvelope), QianjiBpmnWorkflowControlError> {
    let checkpoint_store = resolve_checkpoint_store(service, Some(checkpoint_backend))?
        .ok_or_else(|| io::Error::other("workflow operation requires one checkpoint backend"))?;
    let checkpoint = checkpoint_store.load(instance_id).await?.ok_or_else(|| {
        QianjiBpmnWorkflowControlError::CheckpointMissing {
            instance_id: instance_id.to_string(),
        }
    })?;
    Ok((checkpoint_store, checkpoint))
}
