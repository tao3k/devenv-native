use super::backend::QianjiBpmnCheckpointStore;
use super::error::BpmnOrchestrationError;
use crate::bpmn::{resolve_pending_host_work, resolve_waiting_external_event};
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnCheckpointEnvelope, BpmnHostBridge, BpmnInstanceInit,
    BpmnInstanceState, BpmnPackage, advance_instance, create_instance,
};
use std::sync::Arc;

/// Host-owned BPMN runtime session that keeps one immutable BPMN package and
/// one mutable instance state together.
#[derive(Debug, Clone)]
pub struct QianjiBpmnSession {
    package: Arc<BpmnPackage>,
    instance: BpmnInstanceState,
}

impl QianjiBpmnSession {
    /// Creates one BPMN session from a loaded package plus instance-init input.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the target process does not
    /// exist in the provided package.
    pub fn new(
        package: Arc<BpmnPackage>,
        process_id: &str,
        init: BpmnInstanceInit,
    ) -> Result<Self, BpmnOrchestrationError> {
        let instance = create_instance(package.as_ref(), process_id, init)?;
        Ok(Self { package, instance })
    }

    /// Rebuilds one BPMN session from a checkpoint envelope.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the checkpoint references a
    /// process that does not exist in the loaded package or when the stored
    /// process identity drifts from the current package.
    pub fn from_checkpoint(
        package: Arc<BpmnPackage>,
        checkpoint: BpmnCheckpointEnvelope,
    ) -> Result<Self, BpmnOrchestrationError> {
        validate_checkpoint_process_identity(package.as_ref(), &checkpoint.state)?;
        Ok(Self {
            package,
            instance: checkpoint.state,
        })
    }

    /// Loads one session from the configured checkpoint backend when present.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when checkpoint loading fails or when
    /// the loaded checkpoint drifts from the supplied BPMN package.
    pub async fn load_from_store(
        package: Arc<BpmnPackage>,
        instance_id: &str,
        store: &QianjiBpmnCheckpointStore,
    ) -> Result<Option<Self>, BpmnOrchestrationError> {
        store
            .load(instance_id)
            .await?
            .map(|checkpoint| Self::from_checkpoint(Arc::clone(&package), checkpoint))
            .transpose()
    }

    /// Returns the immutable BPMN package for this session.
    #[must_use]
    pub fn package(&self) -> &BpmnPackage {
        self.package.as_ref()
    }

    /// Returns the current mutable instance state by shared reference.
    #[must_use]
    pub fn instance(&self) -> &BpmnInstanceState {
        &self.instance
    }

    /// Returns the current mutable instance state by mutable reference.
    #[must_use]
    pub fn instance_mut(&mut self) -> &mut BpmnInstanceState {
        &mut self.instance
    }

    /// Returns one versioned checkpoint envelope for the current session state.
    #[must_use]
    pub fn checkpoint(&self) -> BpmnCheckpointEnvelope {
        BpmnCheckpointEnvelope::from_state(self.instance.clone())
    }

    /// Saves the current session checkpoint to the supplied backend.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the checkpoint backend cannot
    /// persist the current session state.
    pub async fn save_checkpoint(
        &self,
        store: &QianjiBpmnCheckpointStore,
    ) -> Result<(), BpmnOrchestrationError> {
        store.save(&self.checkpoint()).await
    }

    /// Saves the current session checkpoint when the caller owns the Valkey
    /// lease for that BPMN instance.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the checkpoint backend cannot
    /// persist the current session state or the caller does not own the lease.
    pub async fn save_checkpoint_as_owner(
        &self,
        store: &QianjiBpmnCheckpointStore,
        owner_token: &str,
    ) -> Result<(), BpmnOrchestrationError> {
        store.save_as_owner(&self.checkpoint(), owner_token).await
    }

    /// Advances the BPMN runtime until the next stable non-host-blocked
    /// outcome.
    ///
    /// This facade keeps pure BPMN semantics inside `qianji-bpmn-engine` while
    /// allowing the host crate to consume one higher-level runtime entrypoint.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the engine rejects the current
    /// runtime state or when the host bridge cannot service pending host work.
    pub async fn run_until_stable<H: BpmnHostBridge>(
        &mut self,
        host: &H,
    ) -> Result<BpmnAdvanceOutcome, BpmnOrchestrationError> {
        let mut outcome = advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
        loop {
            match outcome {
                BpmnAdvanceOutcome::Advanced => {
                    outcome =
                        advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
                }
                BpmnAdvanceOutcome::BlockedOnHost(_) => {
                    outcome =
                        resolve_pending_host_work(self.package.as_ref(), &mut self.instance, host)
                            .await?;
                }
                BpmnAdvanceOutcome::WaitingExternalEvent => {
                    outcome = resolve_waiting_external_event(
                        self.package.as_ref(),
                        &mut self.instance,
                        host,
                    )
                    .await?;
                    if matches!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent) {
                        return Ok(outcome);
                    }
                }
                BpmnAdvanceOutcome::Suspended(_)
                | BpmnAdvanceOutcome::Completed
                | BpmnAdvanceOutcome::Failed(_) => return Ok(outcome),
            }
        }
    }

    /// Splits the session back into its package and instance state.
    #[must_use]
    pub fn into_parts(self) -> (Arc<BpmnPackage>, BpmnInstanceState) {
        (self.package, self.instance)
    }
}

fn validate_checkpoint_process_identity(
    package: &BpmnPackage,
    instance: &BpmnInstanceState,
) -> Result<(), BpmnOrchestrationError> {
    let Some((_, loaded_process)) =
        package.find_process_position(instance.process.process_id.as_ref())
    else {
        return Err(BpmnOrchestrationError::CheckpointProcessMissing {
            process_id: instance.process.process_id.to_string(),
        });
    };

    if loaded_process.key.package_id != instance.process.package_id
        || loaded_process.key.spec_digest_hex != instance.process.spec_digest_hex
    {
        return Err(BpmnOrchestrationError::CheckpointProcessIdentityDrift {
            process_id: instance.process.process_id.to_string(),
            checkpoint_package_id: instance.process.package_id.to_string(),
            checkpoint_spec_digest: instance.process.spec_digest_hex.to_string(),
            loaded_package_id: loaded_process.key.package_id.to_string(),
            loaded_spec_digest: loaded_process.key.spec_digest_hex.to_string(),
        });
    }

    Ok(())
}
