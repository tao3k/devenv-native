use super::checkpoint::load_required_checkpoint;
use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::control::{
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowInterruptReport,
    QianjiBpmnWorkflowInterruptRequest,
};
use crate::bpmn::error::BpmnOrchestrationError;
use crate::bpmn::execution::DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS;
use crate::bpmn::ownership::QianjiBpmnSchedulerLeaseConfig;
use crate::telemetry::unix_millis_now;
use qianji_bpmn_engine::{BpmnCheckpointEnvelope, InstanceLifecycle, SuspendReason};

pub(crate) async fn interrupt_workflow(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowInterruptRequest,
) -> Result<QianjiBpmnWorkflowInterruptReport, QianjiBpmnWorkflowControlError> {
    let (checkpoint_store, mut checkpoint) =
        load_required_checkpoint(service, &request.instance_id, &request.checkpoint_backend)
            .await?;
    mark_checkpoint_interrupted(&mut checkpoint);

    match &request.checkpoint_backend {
        QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey => {
            interrupt_runtime_valkey_checkpoint(service, &checkpoint_store, &checkpoint).await?;
        }
        #[cfg(feature = "duckdb")]
        QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb => {
            checkpoint_store.save(&checkpoint).await?;
        }
    }

    Ok(QianjiBpmnWorkflowInterruptReport {
        checkpoint_store,
        checkpoint_sequence: checkpoint.sequence,
        instance: checkpoint.state,
    })
}

fn mark_checkpoint_interrupted(checkpoint: &mut BpmnCheckpointEnvelope) {
    checkpoint.state.sequence += 1;
    checkpoint.state.lifecycle = InstanceLifecycle::Suspended;
    checkpoint.state.suspend_reason = Some(SuspendReason::HostRequested);
    checkpoint.state.updated_at_ms = unix_millis_now();
    checkpoint.sequence = checkpoint.state.sequence;
}

async fn interrupt_runtime_valkey_checkpoint(
    service: &QianjiBpmnWorkflowControlService,
    checkpoint_store: &QianjiBpmnCheckpointStore,
    checkpoint: &BpmnCheckpointEnvelope,
) -> Result<(), QianjiBpmnWorkflowControlError> {
    let scheduler_identity = service.scheduler_identity.clone().unwrap_or_default();
    let lease = QianjiBpmnSchedulerLeaseConfig::from_scheduler_identity(
        &scheduler_identity,
        DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS,
    )?;
    let owner_token = lease.owner_token().to_string();
    let acquired = checkpoint_store
        .try_acquire_lease(
            checkpoint.state.instance_id.as_ref(),
            owner_token.as_str(),
            lease.lease_ttl_ms(),
        )
        .await?;
    if !acquired {
        return Err(BpmnOrchestrationError::CheckpointLeaseConflict {
            instance_id: checkpoint.state.instance_id.to_string(),
            owner_token,
        }
        .into());
    }

    let save_result = checkpoint_store
        .save_as_owner(checkpoint, lease.owner_token())
        .await;
    let release_result = checkpoint_store
        .release_lease(checkpoint.state.instance_id.as_ref(), lease.owner_token())
        .await;

    save_result?;
    let _released = release_result?;
    Ok(())
}
