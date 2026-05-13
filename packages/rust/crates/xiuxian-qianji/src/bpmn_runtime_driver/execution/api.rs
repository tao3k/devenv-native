use super::{
    QianjiBpmnCheckpointLifecycle, QianjiBpmnExecutionDriver, QianjiBpmnExecutionReport,
    QianjiBpmnExecutionRequest, QianjiBpmnHostCompletionAdvance, QianjiBpmnPendingHostCompletion,
};
use crate::bpmn::error::BpmnOrchestrationError;
use crate::bpmn::ownership::QianjiBpmnSchedulerLeaseConfig;
use crate::bpmn::session::QianjiBpmnSession;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnCheckpointEnvelope, BpmnEngineError, BpmnExecutionTraceEvent,
    BpmnHostBridge, BpmnInstanceInit,
};
use std::sync::Arc;

impl QianjiBpmnExecutionDriver {
    /// Runs the BPMN session until the next stable outcome.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the driver cannot create or
    /// resume the session, when the host cannot service BPMN work, or when the
    /// checkpoint backend fails.
    pub async fn run_until_stable<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.run_with_checkpoint_lifecycle(
            request,
            host,
            QianjiBpmnCheckpointLifecycle::Retain,
            None,
        )
        .await
    }

    pub(in crate::bpmn) async fn run_with_scheduler_lifecycle<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
        checkpoint_lease: Option<&QianjiBpmnSchedulerLeaseConfig>,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.run_with_checkpoint_lifecycle(
            request,
            host,
            QianjiBpmnCheckpointLifecycle::DeleteOnTerminalOutcome,
            checkpoint_lease,
        )
        .await
    }

    pub(in crate::bpmn) async fn run_with_scheduler_lifecycle_and_trace_observer<H, F>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
        checkpoint_lease: Option<&QianjiBpmnSchedulerLeaseConfig>,
        trace_observer: F,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError>
    where
        H: BpmnHostBridge,
        F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
    {
        self.run_with_checkpoint_lifecycle_and_trace_observer(
            request,
            host,
            QianjiBpmnCheckpointLifecycle::DeleteOnTerminalOutcome,
            checkpoint_lease,
            trace_observer,
        )
        .await
    }

    /// Runs the BPMN session until the next stable outcome and reports newly
    /// produced trace events after each runtime step.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the driver cannot create or
    /// resume the session, when the host cannot service BPMN work, or when the
    /// checkpoint backend fails.
    pub async fn run_until_stable_with_trace_observer<H, F>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
        trace_observer: F,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError>
    where
        H: BpmnHostBridge,
        F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
    {
        self.run_with_checkpoint_lifecycle_and_trace_observer(
            request,
            host,
            QianjiBpmnCheckpointLifecycle::Retain,
            None,
            trace_observer,
        )
        .await
    }

    /// Completes one pending host-work item from a checkpointed session, then
    /// advances the BPMN runtime until the next stable outcome.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the checkpoint cannot be
    /// resumed, the engine rejects the explicit host-work result, a later host
    /// bridge operation fails, or checkpoint persistence fails.
    pub async fn complete_pending_host_work_until_stable<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.complete_pending_host_work_with_loaded_checkpoint(
            request,
            None,
            completion,
            host,
            QianjiBpmnHostCompletionAdvance::Stable,
        )
        .await
    }

    /// Completes one pending host-work item from a supplied checkpoint, then
    /// advances the BPMN runtime until the next stable outcome.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the supplied checkpoint cannot
    /// rebuild a session, the engine rejects the explicit host-work result, a
    /// later host bridge operation fails, or checkpoint persistence fails.
    pub async fn complete_pending_host_work_from_checkpoint_until_stable<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        checkpoint: BpmnCheckpointEnvelope,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.complete_pending_host_work_with_loaded_checkpoint(
            request,
            Some(checkpoint),
            completion,
            host,
            QianjiBpmnHostCompletionAdvance::Stable,
        )
        .await
    }

    /// Completes one pending host-work item from a checkpointed session, then
    /// advances the BPMN runtime until the next host boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the checkpoint cannot be
    /// resumed, the engine rejects the explicit host-work result, or
    /// checkpoint persistence fails.
    pub async fn complete_pending_host_work_until_host_boundary<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.complete_pending_host_work_with_loaded_checkpoint(
            request,
            None,
            completion,
            host,
            QianjiBpmnHostCompletionAdvance::HostBoundary,
        )
        .await
    }

    /// Completes one pending host-work item from a supplied checkpoint, then
    /// advances the BPMN runtime until the next host boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the supplied checkpoint cannot
    /// rebuild a session, the engine rejects the explicit host-work result, or
    /// checkpoint persistence fails.
    pub async fn complete_pending_host_work_from_checkpoint_until_host_boundary<
        H: BpmnHostBridge,
    >(
        &self,
        request: &QianjiBpmnExecutionRequest,
        checkpoint: BpmnCheckpointEnvelope,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.complete_pending_host_work_with_loaded_checkpoint(
            request,
            Some(checkpoint),
            completion,
            host,
            QianjiBpmnHostCompletionAdvance::HostBoundary,
        )
        .await
    }

    async fn complete_pending_host_work_with_loaded_checkpoint<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        checkpoint: Option<BpmnCheckpointEnvelope>,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
        advance: QianjiBpmnHostCompletionAdvance,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.acquire_checkpoint_lease_if_needed(request.instance_id.as_str(), None)
            .await?;

        let run_result = async {
            let (mut session, resumed_from_checkpoint) = match checkpoint {
                Some(checkpoint) => (
                    QianjiBpmnSession::from_checkpoint(Arc::clone(&self.package), checkpoint)?,
                    true,
                ),
                None => self.load_or_create_session(request).await?,
            };
            let starting_sequence = session.instance().sequence;
            session.clear_host_requested_interrupt(request.started_at_ms);
            let QianjiBpmnPendingHostCompletion {
                token_id,
                process_id,
                activity_id,
                result,
            } = completion;
            let outcome = match advance {
                QianjiBpmnHostCompletionAdvance::Stable => {
                    session
                        .complete_pending_host_work_until_stable(
                            token_id,
                            process_id.as_str(),
                            activity_id.as_str(),
                            result,
                            host,
                        )
                        .await?
                }
                QianjiBpmnHostCompletionAdvance::HostBoundary => {
                    session
                        .complete_pending_host_work_until_host_boundary(
                            token_id,
                            process_id.as_str(),
                            activity_id.as_str(),
                            result,
                            host,
                        )
                        .await?
                }
                QianjiBpmnHostCompletionAdvance::HumanBoundary => {
                    session
                        .complete_pending_host_work_until_human_boundary(
                            token_id,
                            process_id.as_str(),
                            activity_id.as_str(),
                            result,
                            host,
                        )
                        .await?
                }
            };
            let (checkpoint_saved, checkpoint_deleted) = self
                .finalize_checkpoint(
                    &session,
                    resumed_from_checkpoint,
                    starting_sequence,
                    &outcome,
                    QianjiBpmnCheckpointLifecycle::Retain,
                    None,
                )
                .await?;

            Ok(QianjiBpmnExecutionReport {
                session,
                outcome,
                resumed_from_checkpoint,
                checkpoint_saved,
                checkpoint_deleted,
            })
        }
        .await;

        let release_result = self
            .release_checkpoint_lease_if_needed(request.instance_id.as_str(), None)
            .await;

        match (run_result, release_result) {
            (Err(error), _) | (Ok(_), Err(error)) => Err(error),
            (Ok(report), Ok(())) => Ok(report),
        }
    }

    /// Completes one pending host-work item from a checkpointed session, then
    /// advances through non-human host work until a user/manual boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the checkpoint cannot be resumed,
    /// the explicit result is rejected, a fixture-backed non-human host task
    /// fails, or checkpoint persistence fails.
    pub async fn complete_pending_host_work_until_human_boundary<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.complete_pending_host_work_with_loaded_checkpoint(
            request,
            None,
            completion,
            host,
            QianjiBpmnHostCompletionAdvance::HumanBoundary,
        )
        .await
    }

    /// Completes one pending host-work item from a supplied checkpoint, then
    /// advances through non-human host work until a user/manual boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the supplied checkpoint cannot
    /// rebuild a session, the explicit result is rejected, a fixture-backed
    /// non-human host task fails, or checkpoint persistence fails.
    pub async fn complete_pending_host_work_from_checkpoint_until_human_boundary<
        H: BpmnHostBridge,
    >(
        &self,
        request: &QianjiBpmnExecutionRequest,
        checkpoint: BpmnCheckpointEnvelope,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.complete_pending_host_work_with_loaded_checkpoint(
            request,
            Some(checkpoint),
            completion,
            host,
            QianjiBpmnHostCompletionAdvance::HumanBoundary,
        )
        .await
    }

    /// Runs the BPMN session until the next host boundary or another stable
    /// outcome, then persists the checkpoint when a backend is configured.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the driver cannot create or
    /// resume the session, when the initial host work cannot be serviced, or
    /// when the checkpoint backend fails.
    pub async fn run_until_host_boundary_with_trace_observer<H, F>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
        resolve_initial_host_work: bool,
        trace_observer: F,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError>
    where
        H: BpmnHostBridge,
        F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
    {
        self.acquire_checkpoint_lease_if_needed(request.instance_id.as_str(), None)
            .await?;

        let run_result = async {
            let (mut session, resumed_from_checkpoint) =
                self.load_or_create_session(request).await?;
            let starting_sequence = session.instance().sequence;
            session.clear_host_requested_interrupt(request.started_at_ms);
            let outcome = session
                .run_until_host_boundary_with_trace_observer(
                    host,
                    resolve_initial_host_work,
                    trace_observer,
                )
                .await?;
            let (checkpoint_saved, checkpoint_deleted) = self
                .finalize_checkpoint(
                    &session,
                    resumed_from_checkpoint,
                    starting_sequence,
                    &outcome,
                    QianjiBpmnCheckpointLifecycle::Retain,
                    None,
                )
                .await?;

            Ok(QianjiBpmnExecutionReport {
                session,
                outcome,
                resumed_from_checkpoint,
                checkpoint_saved,
                checkpoint_deleted,
            })
        }
        .await;

        let release_result = self
            .release_checkpoint_lease_if_needed(request.instance_id.as_str(), None)
            .await;

        match (run_result, release_result) {
            (Err(error), _) | (Ok(_), Err(error)) => Err(error),
            (Ok(report), Ok(())) => Ok(report),
        }
    }

    /// Runs the BPMN session through fixture-backed non-human host work until
    /// the next user/manual boundary or another stable outcome.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the driver cannot create or
    /// resume the session, when non-human host work cannot be serviced, or when
    /// checkpoint persistence fails.
    pub async fn run_until_human_boundary_with_trace_observer<H, F>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
        resolve_initial_host_work: bool,
        trace_observer: F,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError>
    where
        H: BpmnHostBridge,
        F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
    {
        self.acquire_checkpoint_lease_if_needed(request.instance_id.as_str(), None)
            .await?;

        let run_result = async {
            let (mut session, resumed_from_checkpoint) =
                self.load_or_create_session(request).await?;
            let starting_sequence = session.instance().sequence;
            session.clear_host_requested_interrupt(request.started_at_ms);
            let outcome = session
                .run_until_human_boundary_with_trace_observer(
                    host,
                    resolve_initial_host_work,
                    trace_observer,
                )
                .await?;
            let (checkpoint_saved, checkpoint_deleted) = self
                .finalize_checkpoint(
                    &session,
                    resumed_from_checkpoint,
                    starting_sequence,
                    &outcome,
                    QianjiBpmnCheckpointLifecycle::Retain,
                    None,
                )
                .await?;

            Ok(QianjiBpmnExecutionReport {
                session,
                outcome,
                resumed_from_checkpoint,
                checkpoint_saved,
                checkpoint_deleted,
            })
        }
        .await;

        let release_result = self
            .release_checkpoint_lease_if_needed(request.instance_id.as_str(), None)
            .await;

        match (run_result, release_result) {
            (Err(error), _) | (Ok(_), Err(error)) => Err(error),
            (Ok(report), Ok(())) => Ok(report),
        }
    }

    async fn run_with_checkpoint_lifecycle<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
        checkpoint_lifecycle: QianjiBpmnCheckpointLifecycle,
        checkpoint_lease: Option<&QianjiBpmnSchedulerLeaseConfig>,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.acquire_checkpoint_lease_if_needed(request.instance_id.as_str(), checkpoint_lease)
            .await?;

        let run_result = async {
            let (mut session, resumed_from_checkpoint) =
                self.load_or_create_session(request).await?;
            let starting_sequence = session.instance().sequence;
            session.clear_host_requested_interrupt(request.started_at_ms);
            let outcome = session.run_until_stable(host).await?;
            let (checkpoint_saved, checkpoint_deleted) = self
                .finalize_checkpoint(
                    &session,
                    resumed_from_checkpoint,
                    starting_sequence,
                    &outcome,
                    checkpoint_lifecycle,
                    checkpoint_lease,
                )
                .await?;

            Ok(QianjiBpmnExecutionReport {
                session,
                outcome,
                resumed_from_checkpoint,
                checkpoint_saved,
                checkpoint_deleted,
            })
        }
        .await;

        let release_result = self
            .release_checkpoint_lease_if_needed(request.instance_id.as_str(), checkpoint_lease)
            .await;

        match (run_result, release_result) {
            (Err(error), _) | (Ok(_), Err(error)) => Err(error),
            (Ok(report), Ok(())) => Ok(report),
        }
    }

    async fn run_with_checkpoint_lifecycle_and_trace_observer<H, F>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
        checkpoint_lifecycle: QianjiBpmnCheckpointLifecycle,
        checkpoint_lease: Option<&QianjiBpmnSchedulerLeaseConfig>,
        trace_observer: F,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError>
    where
        H: BpmnHostBridge,
        F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
    {
        self.acquire_checkpoint_lease_if_needed(request.instance_id.as_str(), checkpoint_lease)
            .await?;

        let run_result = async {
            let (mut session, resumed_from_checkpoint) =
                self.load_or_create_session(request).await?;
            let starting_sequence = session.instance().sequence;
            session.clear_host_requested_interrupt(request.started_at_ms);
            let outcome = session
                .run_until_stable_with_trace_observer(host, trace_observer)
                .await?;
            let (checkpoint_saved, checkpoint_deleted) = self
                .finalize_checkpoint(
                    &session,
                    resumed_from_checkpoint,
                    starting_sequence,
                    &outcome,
                    checkpoint_lifecycle,
                    checkpoint_lease,
                )
                .await?;

            Ok(QianjiBpmnExecutionReport {
                session,
                outcome,
                resumed_from_checkpoint,
                checkpoint_saved,
                checkpoint_deleted,
            })
        }
        .await;

        let release_result = self
            .release_checkpoint_lease_if_needed(request.instance_id.as_str(), checkpoint_lease)
            .await;

        match (run_result, release_result) {
            (Err(error), _) | (Ok(_), Err(error)) => Err(error),
            (Ok(report), Ok(())) => Ok(report),
        }
    }

    pub(super) async fn load_or_create_session(
        &self,
        request: &QianjiBpmnExecutionRequest,
    ) -> Result<(QianjiBpmnSession, bool), BpmnOrchestrationError> {
        if let Some(start_at_node_id) = request.start_at_node_id.as_deref() {
            return self
                .create_start_at_session(request, start_at_node_id)
                .await
                .map(|session| (session, false));
        }

        if let Some(store) = self.checkpoint_store.as_ref()
            && let Some(session) = QianjiBpmnSession::load_from_store(
                Arc::clone(&self.package),
                &request.instance_id,
                store,
            )
            .await?
        {
            return Ok((session, true));
        }

        let Some(initial_variables) = request.initial_variables.clone() else {
            return Err(BpmnOrchestrationError::FreshContextRequired {
                process_id: request.process_id.to_string().into(),
                instance_id: request.instance_id.to_string().into(),
            });
        };

        let session = QianjiBpmnSession::new(
            Arc::clone(&self.package),
            request.process_id.as_str(),
            BpmnInstanceInit::new(
                request.instance_id.as_str(),
                initial_variables,
                request.started_at_ms,
            ),
        )?;
        Ok((session, false))
    }

    pub(super) async fn create_start_at_session(
        &self,
        request: &QianjiBpmnExecutionRequest,
        start_at_node_id: &str,
    ) -> Result<QianjiBpmnSession, BpmnOrchestrationError> {
        if let Some(store) = self.checkpoint_store.as_ref()
            && store.load(request.instance_id.as_str()).await?.is_some()
        {
            return Err(BpmnOrchestrationError::StartAtCheckpointExists {
                instance_id: request.instance_id.clone(),
            });
        }

        let Some(initial_variables) = request.initial_variables.clone() else {
            return Err(BpmnOrchestrationError::FreshContextRequired {
                process_id: request.process_id.to_string().into(),
                instance_id: request.instance_id.to_string().into(),
            });
        };

        QianjiBpmnSession::new_at_node(
            Arc::clone(&self.package),
            request.process_id.as_str(),
            BpmnInstanceInit::new(
                request.instance_id.as_str(),
                initial_variables,
                request.started_at_ms,
            ),
            start_at_node_id,
        )
    }

    pub(super) async fn finalize_checkpoint(
        &self,
        session: &QianjiBpmnSession,
        resumed_from_checkpoint: bool,
        starting_sequence: u64,
        outcome: &BpmnAdvanceOutcome,
        checkpoint_lifecycle: QianjiBpmnCheckpointLifecycle,
        checkpoint_lease: Option<&QianjiBpmnSchedulerLeaseConfig>,
    ) -> Result<(bool, bool), BpmnOrchestrationError> {
        let Some(store) = self.checkpoint_store.as_ref() else {
            return Ok((false, false));
        };

        self.renew_checkpoint_lease_if_needed(
            session.instance().instance_id.as_ref(),
            checkpoint_lease,
            matches!(
                outcome,
                BpmnAdvanceOutcome::Completed
                    | BpmnAdvanceOutcome::Failed(_)
                    | BpmnAdvanceOutcome::WaitingExternalEvent
                    | BpmnAdvanceOutcome::Suspended(_)
            ) && !(resumed_from_checkpoint && session.instance().sequence <= starting_sequence),
        )
        .await?;

        if checkpoint_lifecycle == QianjiBpmnCheckpointLifecycle::DeleteOnTerminalOutcome
            && matches!(
                outcome,
                BpmnAdvanceOutcome::Completed | BpmnAdvanceOutcome::Failed(_)
            )
        {
            if let Some(checkpoint_lease) = checkpoint_lease {
                store
                    .delete_as_owner(
                        session.instance().instance_id.as_ref(),
                        checkpoint_lease.owner_token(),
                    )
                    .await?;
            } else {
                store
                    .delete(session.instance().instance_id.as_ref())
                    .await?;
            }
            return Ok((false, true));
        }

        if resumed_from_checkpoint && session.instance().sequence <= starting_sequence {
            return Ok((false, false));
        }

        if let Some(checkpoint_lease) = checkpoint_lease {
            session
                .save_checkpoint_as_owner(store, checkpoint_lease.owner_token())
                .await?;
        } else {
            session.save_checkpoint(store).await?;
        }
        Ok((true, false))
    }

    pub(super) async fn acquire_checkpoint_lease_if_needed(
        &self,
        instance_id: &str,
        checkpoint_lease: Option<&QianjiBpmnSchedulerLeaseConfig>,
    ) -> Result<(), BpmnOrchestrationError> {
        let Some(checkpoint_lease) = checkpoint_lease else {
            return Ok(());
        };

        let Some(store) = self.checkpoint_store.as_ref() else {
            return Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: "none".to_string(),
            });
        };

        let acquired = store
            .try_acquire_lease(
                instance_id,
                checkpoint_lease.owner_token(),
                checkpoint_lease.lease_ttl_ms(),
            )
            .await?;
        if !acquired {
            return Err(BpmnOrchestrationError::CheckpointLeaseConflict {
                instance_id: instance_id.into(),
                owner_token: checkpoint_lease.owner_token().into(),
            });
        }
        Ok(())
    }

    pub(super) async fn renew_checkpoint_lease_if_needed(
        &self,
        instance_id: &str,
        checkpoint_lease: Option<&QianjiBpmnSchedulerLeaseConfig>,
        should_renew: bool,
    ) -> Result<(), BpmnOrchestrationError> {
        let Some(checkpoint_lease) = checkpoint_lease else {
            return Ok(());
        };
        if !should_renew {
            return Ok(());
        }

        let store = self.checkpoint_store.as_ref().ok_or_else(|| {
            BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: "none".to_string(),
            }
        })?;
        let renewed = store
            .renew_lease(
                instance_id,
                checkpoint_lease.owner_token(),
                checkpoint_lease.lease_ttl_ms(),
            )
            .await?;
        if !renewed {
            return Err(BpmnEngineError::CheckpointLeaseNotOwned {
                instance_id: instance_id.to_string().into(),
            }
            .into());
        }
        Ok(())
    }

    pub(super) async fn release_checkpoint_lease_if_needed(
        &self,
        instance_id: &str,
        checkpoint_lease: Option<&QianjiBpmnSchedulerLeaseConfig>,
    ) -> Result<(), BpmnOrchestrationError> {
        let Some(checkpoint_lease) = checkpoint_lease else {
            return Ok(());
        };

        let store = self.checkpoint_store.as_ref().ok_or_else(|| {
            BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: "none".to_string(),
            }
        })?;
        let released = store
            .release_lease(instance_id, checkpoint_lease.owner_token())
            .await?;
        if !released {
            return Err(BpmnEngineError::CheckpointLeaseNotOwned {
                instance_id: instance_id.to_string().into(),
            }
            .into());
        }
        Ok(())
    }
}
