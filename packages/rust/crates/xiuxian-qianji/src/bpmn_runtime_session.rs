//! Bpmn runtime session surface for `xiuxian-qianji`.

use super::backend::QianjiBpmnCheckpointStore;
use super::error::{BpmnOrchestrationError, BpmnUnsupportedStartNodeKind};
use crate::bpmn::{resolve_pending_host_work, resolve_waiting_external_event};
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnCheckpointEnvelope, BpmnExecutionTraceEvent, BpmnHostBridge,
    BpmnInstanceInit, BpmnInstanceState, BpmnNodeKind, BpmnPackage, InstanceLifecycle,
    NodeRuntimeStatus, PendingHostWork, PendingHostWorkApplyInput, PendingHostWorkKind,
    PendingHostWorkResult, SuspendReason, TokenRecord, advance_instance,
    apply_pending_host_work_result, create_instance,
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
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    pub fn new(
        package: Arc<BpmnPackage>,
        process_id: &str,
        init: BpmnInstanceInit,
    ) -> Result<Self, BpmnOrchestrationError> {
        let instance = create_instance(package.as_ref(), process_id, init)?;
        Ok(Self { package, instance })
    }

    pub(crate) fn new_at_node(
        package: Arc<BpmnPackage>,
        process_id: &str,
        init: BpmnInstanceInit,
        node_id: &str,
    ) -> Result<Self, BpmnOrchestrationError> {
        let process = package.find_process(process_id).ok_or_else(|| {
            BpmnOrchestrationError::StartAtNodeMissing {
                process_id: process_id.into(),
                node_id: node_id.into(),
            }
        })?;
        let node = process
            .nodes
            .iter()
            .find(|node| node.bpmn_id.as_ref() == node_id)
            .ok_or_else(|| BpmnOrchestrationError::StartAtNodeMissing {
                process_id: process_id.into(),
                node_id: node_id.into(),
            })?;
        if !start_at_node_kind_is_supported(&node.kind) {
            return Err(BpmnOrchestrationError::StartAtNodeUnsupported {
                process_id: process_id.into(),
                node_id: node_id.into(),
                node_kind: BpmnUnsupportedStartNodeKind::new(start_at_node_kind_label(&node.kind)),
            });
        }
        let node_index = node.index;

        let mut session = Self::new(package, process_id, init)?;
        session.instance.next_token_id = 2;
        session.instance.active_tokens.push(TokenRecord {
            token_id: 1,
            node_index,
            incoming_edge_index: None,
            inclusive_join_hint: None,
        });
        if let Some(node_state) = session.instance.node_states.get_mut(node_index as usize) {
            node_state.status = NodeRuntimeStatus::Queued;
        }
        Ok(session)
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
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
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

    /// Clears a host-requested interrupt marker before an explicit resume.
    ///
    /// Returns `true` when the stored instance was changed.
    pub(crate) fn clear_host_requested_interrupt(&mut self, now_ms: u64) -> bool {
        if !matches!(self.instance.lifecycle, InstanceLifecycle::Suspended)
            || !matches!(
                self.instance.suspend_reason,
                Some(SuspendReason::HostRequested)
            )
        {
            return false;
        }

        self.instance.lifecycle = InstanceLifecycle::Waiting;
        self.instance.suspend_reason = None;
        self.instance.sequence += 1;
        self.instance.updated_at_ms = now_ms;
        true
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

    /// Applies one explicit pending host-work result and advances the BPMN
    /// runtime until the next stable outcome.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the engine rejects the pending
    /// host-work result, when the host bridge cannot service later work, or
    /// when event polling fails.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    /// Positional boundary: this compatibility API keeps the established public call shape.
    pub async fn complete_pending_host_work_until_stable<H: BpmnHostBridge>(
        &mut self,
        token_id: u64,
        process_id: &str,
        activity_id: &str,
        result: PendingHostWorkResult,
        host: &H,
    ) -> Result<BpmnAdvanceOutcome, BpmnOrchestrationError> {
        validate_pending_host_work_identity(
            self.package.as_ref(),
            &self.instance,
            token_id,
            process_id,
            activity_id,
        )?;
        let completed_at_ms = self.instance.updated_at_ms;
        let mut outcome = apply_pending_host_work_result(PendingHostWorkApplyInput {
            package: self.package.as_ref(),
            instance: &mut self.instance,
            token_id: token_id.into(),
            result,
            completed_at_ms: completed_at_ms.into(),
        })?;
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

    /// Applies one explicit pending host-work result and advances until the
    /// next host boundary or another stable terminal/waiting outcome.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the engine rejects the pending
    /// host-work result or the runtime state.
    /// Positional boundary: this compatibility API keeps the established public call shape.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    pub async fn complete_pending_host_work_until_host_boundary<H: BpmnHostBridge>(
        &mut self,
        token_id: u64,
        process_id: &str,
        activity_id: &str,
        result: PendingHostWorkResult,
        host: &H,
    ) -> Result<BpmnAdvanceOutcome, BpmnOrchestrationError> {
        validate_pending_host_work_identity(
            self.package.as_ref(),
            &self.instance,
            token_id,
            process_id,
            activity_id,
        )?;
        let completed_at_ms = self.instance.updated_at_ms;
        let mut outcome = apply_pending_host_work_result(PendingHostWorkApplyInput {
            package: self.package.as_ref(),
            instance: &mut self.instance,
            token_id: token_id.into(),
            result,
            completed_at_ms: completed_at_ms.into(),
        })?;
        loop {
            match outcome {
                BpmnAdvanceOutcome::Advanced => {
                    outcome =
                        advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
                }
                BpmnAdvanceOutcome::BlockedOnHost(_)
                | BpmnAdvanceOutcome::WaitingExternalEvent
                | BpmnAdvanceOutcome::Suspended(_)
                | BpmnAdvanceOutcome::Completed
                | BpmnAdvanceOutcome::Failed(_) => return Ok(outcome),
            }
        }
    }

    /// Applies one explicit pending host-work result and advances through
    /// fixture-backed non-human host work until the next human boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the engine rejects the explicit
    /// result or when a non-human host task cannot be resolved by the supplied
    /// host bridge.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    /// Positional boundary: this compatibility API keeps the established public call shape.
    pub async fn complete_pending_host_work_until_human_boundary<H: BpmnHostBridge>(
        &mut self,
        token_id: u64,
        process_id: &str,
        activity_id: &str,
        result: PendingHostWorkResult,
        host: &H,
    ) -> Result<BpmnAdvanceOutcome, BpmnOrchestrationError> {
        validate_pending_host_work_identity(
            self.package.as_ref(),
            &self.instance,
            token_id,
            process_id,
            activity_id,
        )?;
        let completed_at_ms = self.instance.updated_at_ms;
        let mut outcome = apply_pending_host_work_result(PendingHostWorkApplyInput {
            package: self.package.as_ref(),
            instance: &mut self.instance,
            token_id: token_id.into(),
            result,
            completed_at_ms: completed_at_ms.into(),
        })?;
        loop {
            match outcome {
                BpmnAdvanceOutcome::Advanced => {
                    outcome =
                        advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
                }
                BpmnAdvanceOutcome::BlockedOnHost(pending) => {
                    if pending_host_work_contains_human_boundary(&pending) {
                        return Ok(BpmnAdvanceOutcome::BlockedOnHost(pending));
                    }
                    outcome =
                        resolve_pending_host_work(self.package.as_ref(), &mut self.instance, host)
                            .await?;
                }
                BpmnAdvanceOutcome::WaitingExternalEvent
                | BpmnAdvanceOutcome::Suspended(_)
                | BpmnAdvanceOutcome::Completed
                | BpmnAdvanceOutcome::Failed(_) => return Ok(outcome),
            }
        }
    }

    /// Advances the BPMN runtime until the next stable non-host-blocked
    /// outcome while reporting newly produced execution trace events after
    /// each runtime step.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the engine rejects the current
    /// runtime state or when the host bridge cannot service pending host work.
    pub async fn run_until_stable_with_trace_observer<H, F>(
        &mut self,
        host: &H,
        mut trace_observer: F,
    ) -> Result<BpmnAdvanceOutcome, BpmnOrchestrationError>
    where
        H: BpmnHostBridge,
        F: FnMut(&Self, &[BpmnExecutionTraceEvent]),
    {
        let mut next_trace_index = self.instance.trace.len();
        let mut outcome = advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
        self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
        loop {
            match outcome {
                BpmnAdvanceOutcome::Advanced => {
                    outcome =
                        advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
                    self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
                }
                BpmnAdvanceOutcome::BlockedOnHost(_) => {
                    outcome =
                        resolve_pending_host_work(self.package.as_ref(), &mut self.instance, host)
                            .await?;
                    self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
                }
                BpmnAdvanceOutcome::WaitingExternalEvent => {
                    outcome = resolve_waiting_external_event(
                        self.package.as_ref(),
                        &mut self.instance,
                        host,
                    )
                    .await?;
                    self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
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

    /// Advances the BPMN runtime until it reaches a host boundary or another
    /// stable outcome, while reporting newly produced execution trace events.
    ///
    /// When `resolve_initial_host_work` is true, any pending host work already
    /// present in the loaded checkpoint is resolved through the supplied host
    /// before the runtime advances to the next boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the engine rejects the current
    /// runtime state or when the host bridge cannot service the initial pending
    /// host work.
    pub async fn run_until_host_boundary_with_trace_observer<H, F>(
        &mut self,
        host: &H,
        resolve_initial_host_work: bool,
        mut trace_observer: F,
    ) -> Result<BpmnAdvanceOutcome, BpmnOrchestrationError>
    where
        H: BpmnHostBridge,
        F: FnMut(&Self, &[BpmnExecutionTraceEvent]),
    {
        let mut next_trace_index = self.instance.trace.len();
        let mut outcome = if resolve_initial_host_work
            && !self.instance.pending_host_work.is_empty()
        {
            let outcome =
                resolve_pending_host_work(self.package.as_ref(), &mut self.instance, host).await?;
            self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
            outcome
        } else {
            let outcome = advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
            self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
            outcome
        };

        loop {
            match outcome {
                BpmnAdvanceOutcome::Advanced => {
                    outcome =
                        advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
                    self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
                }
                BpmnAdvanceOutcome::BlockedOnHost(_)
                | BpmnAdvanceOutcome::WaitingExternalEvent
                | BpmnAdvanceOutcome::Suspended(_)
                | BpmnAdvanceOutcome::Completed
                | BpmnAdvanceOutcome::Failed(_) => return Ok(outcome),
            }
        }
    }

    /// Advances the BPMN runtime through non-human host work until it reaches
    /// a user/manual boundary or another stable terminal/waiting outcome.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the engine rejects the current
    /// runtime state or when non-human host work cannot be resolved by the
    /// supplied host bridge.
    pub async fn run_until_human_boundary_with_trace_observer<H, F>(
        &mut self,
        host: &H,
        resolve_initial_host_work: bool,
        mut trace_observer: F,
    ) -> Result<BpmnAdvanceOutcome, BpmnOrchestrationError>
    where
        H: BpmnHostBridge,
        F: FnMut(&Self, &[BpmnExecutionTraceEvent]),
    {
        let mut next_trace_index = self.instance.trace.len();
        let mut outcome = if resolve_initial_host_work
            && !self.instance.pending_host_work.is_empty()
        {
            if pending_host_work_contains_human_boundary(&self.instance.pending_host_work) {
                return Ok(BpmnAdvanceOutcome::BlockedOnHost(
                    self.instance.pending_host_work.clone(),
                ));
            }
            let outcome =
                resolve_pending_host_work(self.package.as_ref(), &mut self.instance, host).await?;
            self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
            outcome
        } else {
            let outcome = advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
            self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
            outcome
        };

        loop {
            match outcome {
                BpmnAdvanceOutcome::Advanced => {
                    outcome =
                        advance_instance(self.package.as_ref(), &mut self.instance, host).await?;
                    self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
                }
                BpmnAdvanceOutcome::BlockedOnHost(pending) => {
                    if pending_host_work_contains_human_boundary(&pending) {
                        return Ok(BpmnAdvanceOutcome::BlockedOnHost(pending));
                    }
                    outcome =
                        resolve_pending_host_work(self.package.as_ref(), &mut self.instance, host)
                            .await?;
                    self.emit_new_trace_events(&mut next_trace_index, &mut trace_observer);
                }
                BpmnAdvanceOutcome::WaitingExternalEvent
                | BpmnAdvanceOutcome::Suspended(_)
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

    fn emit_new_trace_events<F>(&self, next_trace_index: &mut usize, trace_observer: &mut F)
    where
        F: FnMut(&Self, &[BpmnExecutionTraceEvent]),
    {
        if *next_trace_index >= self.instance.trace.len() {
            return;
        }
        let events = self.instance.trace[*next_trace_index..].to_vec();
        *next_trace_index = self.instance.trace.len();
        trace_observer(self, &events);
    }
}

fn pending_host_work_contains_human_boundary(pending: &[PendingHostWork]) -> bool {
    pending.iter().any(|work| {
        matches!(
            work.kind,
            PendingHostWorkKind::User | PendingHostWorkKind::Manual
        )
    })
}

fn validate_pending_host_work_identity(
    package: &BpmnPackage,
    instance: &BpmnInstanceState,
    token_id: u64,
    expected_process_id: &str,
    expected_activity_id: &str,
) -> Result<(), BpmnOrchestrationError> {
    let pending = instance
        .pending_host_work
        .iter()
        .find(|work| work.token_id == token_id)
        .ok_or_else(
            || qianji_bpmn_engine::BpmnEngineError::MissingPendingHostWorkToken {
                instance_id: instance.instance_id.to_string().into(),
                token_id: token_id.into(),
            },
        )?;

    let actual_process_id = pending
        .process_id
        .as_deref()
        .unwrap_or(instance.process.process_id.as_ref());
    let actual_activity_id = pending
        .activity_id
        .as_deref()
        .or_else(|| {
            package.find_process(actual_process_id).and_then(|process| {
                process
                    .nodes
                    .get(pending.node_index as usize)
                    .map(|node| node.bpmn_id.as_ref())
            })
        })
        .unwrap_or("<missing>");

    if actual_process_id == expected_process_id && actual_activity_id == expected_activity_id {
        return Ok(());
    }

    Err(BpmnOrchestrationError::pending_host_work_identity_mismatch(
        instance.instance_id.to_string(),
        token_id,
        expected_process_id.to_string(),
        expected_activity_id.to_string(),
        actual_process_id.to_string(),
        actual_activity_id.to_string(),
    ))
}

fn start_at_node_kind_is_supported(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::SendTask
            | BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
    )
}

fn start_at_node_kind_label(kind: &BpmnNodeKind) -> &'static str {
    match kind {
        BpmnNodeKind::StartEvent => "start_event",
        BpmnNodeKind::EndEvent => "end_event",
        BpmnNodeKind::IntermediateThrowEvent => "intermediate_throw_event",
        BpmnNodeKind::IntermediateCatchEvent => "intermediate_catch_event",
        BpmnNodeKind::BoundaryEvent => "boundary_event",
        BpmnNodeKind::SendTask => "send_task",
        BpmnNodeKind::ServiceTask => "service_task",
        BpmnNodeKind::ScriptTask => "script_task",
        BpmnNodeKind::UserTask => "user_task",
        BpmnNodeKind::ManualTask => "manual_task",
        BpmnNodeKind::BusinessRuleTask => "business_rule_task",
        BpmnNodeKind::Gateway => "gateway",
        BpmnNodeKind::SubProcess => "sub_process",
        BpmnNodeKind::ReceiveTask => "receive_task",
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
            process_id: instance.process.process_id.as_ref().into(),
        });
    };

    if loaded_process.key.package_id != instance.process.package_id
        || loaded_process.key.spec_digest_hex != instance.process.spec_digest_hex
    {
        return Err(BpmnOrchestrationError::CheckpointProcessIdentityDrift {
            process_id: instance.process.process_id.as_ref().into(),
            checkpoint_package_id: instance.process.package_id.as_ref().into(),
            checkpoint_spec_digest: instance.process.spec_digest_hex.to_string(),
            loaded_package_id: loaded_process.key.package_id.as_ref().into(),
            loaded_spec_digest: loaded_process.key.spec_digest_hex.to_string(),
        });
    }

    Ok(())
}
