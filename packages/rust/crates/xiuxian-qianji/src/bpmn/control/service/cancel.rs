use super::checkpoint::load_required_checkpoint;
use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::control::{
    QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowCancelRequest,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService,
};
use crate::bpmn::error::BpmnOrchestrationError;
use crate::bpmn::execution::DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS;
use crate::bpmn::ownership::QianjiBpmnSchedulerLeaseConfig;

pub(crate) async fn cancel_workflow(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowCancelRequest,
) -> Result<QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowControlError> {
    let (checkpoint_store, checkpoint) =
        load_required_checkpoint(service, &request.instance_id, &request.checkpoint_backend)
            .await?;

    match &request.checkpoint_backend {
        QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey => {
            cancel_runtime_valkey_checkpoint(service, &checkpoint_store, &request.instance_id)
                .await?;
        }
        #[cfg(feature = "duckdb")]
        QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb => {
            checkpoint_store.delete(&request.instance_id).await?;
        }
    }

    Ok(QianjiBpmnWorkflowCancelReport {
        checkpoint_store,
        checkpoint_sequence: checkpoint.sequence,
        instance: checkpoint.state,
    })
}

async fn cancel_runtime_valkey_checkpoint(
    service: &QianjiBpmnWorkflowControlService,
    checkpoint_store: &QianjiBpmnCheckpointStore,
    instance_id: &str,
) -> Result<(), QianjiBpmnWorkflowControlError> {
    let scheduler_identity = service.scheduler_identity.clone().unwrap_or_default();
    let lease = QianjiBpmnSchedulerLeaseConfig::from_scheduler_identity(
        &scheduler_identity,
        DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS,
    )?;
    let owner_token = lease.owner_token().to_string();
    let acquired = checkpoint_store
        .try_acquire_lease(instance_id, owner_token.as_str(), lease.lease_ttl_ms())
        .await?;
    if !acquired {
        return Err(BpmnOrchestrationError::CheckpointLeaseConflict {
            instance_id: instance_id.into(),
            owner_token: owner_token.into(),
        }
        .into());
    }

    let delete_result = checkpoint_store
        .delete_as_owner(instance_id, lease.owner_token())
        .await;
    let release_result = checkpoint_store
        .release_lease(instance_id, lease.owner_token())
        .await;

    delete_result?;
    let _released = release_result?;
    Ok(())
}
