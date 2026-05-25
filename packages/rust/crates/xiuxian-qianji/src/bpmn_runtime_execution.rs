//! Bpmn runtime execution surface for `xiuxian-qianji`.

use super::backend::QianjiBpmnCheckpointStore;
use super::driver::{
    QianjiBpmnExecutionDriver, QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest,
    QianjiBpmnPendingHostCompletion,
};
use super::error::BpmnOrchestrationError;
use super::scheduler::QianjiBpmnExecutionScheduler;
use super::session::QianjiBpmnSession;
use crate::scheduler_identity::SchedulerAgentIdentity;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnExecutionTraceEvent, BpmnHostBridge, BpmnPackage,
};

/// Default lease TTL used when the host runtime enables scheduler-owned BPMN
/// checkpoint lifecycle from `SchedulerAgentIdentity`.
pub const DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS: u64 = 30_000;

/// Concrete host-owned BPMN execution mode selected for one bounded run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QianjiBpmnExecutionMode {
    /// Plain checkpoint-aware driver behavior without scheduler-owned terminal
    /// cleanup or lease ownership.
    Driver,
    /// BPMN-specific scheduler lifecycle with terminal checkpoint cleanup and
    /// optional Valkey-backed single-writer lease ownership.
    SchedulerLifecycle,
}

/// Host-owned BPMN execution facade that selects between the direct driver and
/// the BPMN-specific scheduler lifecycle.
#[derive(Debug, Clone)]
pub struct QianjiBpmnExecutionFacade {
    package: Arc<BpmnPackage>,
    checkpoint_store: Option<QianjiBpmnCheckpointStore>,
    scheduler_identity: Option<SchedulerAgentIdentity>,
    scheduler_lease_ttl_ms: u64,
}

impl QianjiBpmnExecutionFacade {
    /// Creates one execution facade from a loaded BPMN package plus optional
    /// checkpoint storage.
    #[must_use]
    pub fn new(
        package: Arc<BpmnPackage>,
        checkpoint_store: Option<QianjiBpmnCheckpointStore>,
    ) -> Self {
        Self {
            package,
            checkpoint_store,
            scheduler_identity: None,
            scheduler_lease_ttl_ms: DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS,
        }
    }

    /// Records one scheduler execution identity that may enable the BPMN
    /// scheduler-owned lifecycle on the Valkey-backed path.
    #[must_use]
    pub fn with_scheduler_identity(mut self, scheduler_identity: SchedulerAgentIdentity) -> Self {
        self.scheduler_identity = Some(scheduler_identity);
        self
    }

    /// Overrides the scheduler lease TTL used when the scheduler-owned
    /// lifecycle is selected.
    #[must_use]
    pub fn with_scheduler_lease_ttl_ms(mut self, scheduler_lease_ttl_ms: u64) -> Self {
        self.scheduler_lease_ttl_ms = scheduler_lease_ttl_ms;
        self
    }

    /// Returns the execution mode currently selected by the facade.
    #[must_use]
    pub fn execution_mode(&self) -> QianjiBpmnExecutionMode {
        if self.should_use_scheduler_lifecycle() {
            QianjiBpmnExecutionMode::SchedulerLifecycle
        } else {
            QianjiBpmnExecutionMode::Driver
        }
    }

    /// Runs the BPMN request through the selected host-owned execution path.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the selected driver or
    /// scheduler-owned lifecycle cannot create, resume, or advance the BPMN
    /// session, or when checkpoint persistence fails.
    pub async fn run<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        match self.build_scheduler()? {
            Some(scheduler) => scheduler.run(request, host).await,
            None => self.driver().run_until_stable(request, host).await,
        }
    }

    /// Runs the BPMN request through the selected host-owned execution path
    /// while reporting newly produced trace events after each runtime step.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the selected driver or
    /// scheduler-owned lifecycle cannot create, resume, or advance the BPMN
    /// session, or when checkpoint persistence fails.
    pub async fn run_with_trace_observer<H, F>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
        trace_observer: F,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError>
    where
        H: BpmnHostBridge,
        F: FnMut(&QianjiBpmnSession, &[BpmnExecutionTraceEvent]),
    {
        match self.build_scheduler()? {
            Some(scheduler) => {
                scheduler
                    .run_with_trace_observer(request, host, trace_observer)
                    .await
            }
            None => {
                self.driver()
                    .run_until_stable_with_trace_observer(request, host, trace_observer)
                    .await
            }
        }
    }

    /// Completes one explicit pending host-work result through the direct
    /// checkpoint-aware driver.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the checkpoint cannot be
    /// resumed, the explicit result is rejected, later host work fails, or the
    /// checkpoint backend cannot persist the resulting state.
    pub async fn complete_pending_host_work<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.driver()
            .complete_pending_host_work_until_stable(request, completion, host)
            .await
    }

    /// Completes one explicit pending host-work result from a supplied
    /// checkpoint envelope through the direct checkpoint-aware driver.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the supplied checkpoint cannot
    /// rebuild a session, the explicit result is rejected, later host work
    /// fails, or the checkpoint backend cannot persist the resulting state.
    pub async fn complete_pending_host_work_from_checkpoint<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        checkpoint: BpmnCheckpointEnvelope,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.driver()
            .complete_pending_host_work_from_checkpoint_until_stable(
                request, checkpoint, completion, host,
            )
            .await
    }

    /// Completes one explicit pending host-work result, then stops at the next
    /// host boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the checkpoint cannot be
    /// resumed, the explicit result is rejected, or the checkpoint backend
    /// cannot persist the resulting state.
    pub async fn complete_pending_host_work_until_host_boundary<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.driver()
            .complete_pending_host_work_until_host_boundary(request, completion, host)
            .await
    }

    /// Completes one explicit pending host-work result from a supplied
    /// checkpoint envelope, then stops at the next host boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the supplied checkpoint cannot
    /// rebuild a session, the explicit result is rejected, or the checkpoint
    /// backend cannot persist the resulting state.
    pub async fn complete_pending_host_work_from_checkpoint_until_host_boundary<
        H: BpmnHostBridge,
    >(
        &self,
        request: &QianjiBpmnExecutionRequest,
        checkpoint: BpmnCheckpointEnvelope,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.driver()
            .complete_pending_host_work_from_checkpoint_until_host_boundary(
                request, checkpoint, completion, host,
            )
            .await
    }

    /// Completes one explicit pending host-work result, then continues through
    /// non-human host work until the next user/manual boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the checkpoint cannot be resumed,
    /// the explicit result is rejected, later non-human host work fails, or the
    /// checkpoint backend cannot persist the resulting state.
    pub async fn complete_pending_host_work_until_human_boundary<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.driver()
            .complete_pending_host_work_until_human_boundary(request, completion, host)
            .await
    }

    /// Completes one explicit pending host-work result from a supplied
    /// checkpoint envelope, then continues through non-human host work until
    /// the next user/manual boundary.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the supplied checkpoint cannot
    /// rebuild a session, the explicit result is rejected, later non-human
    /// host work fails, or the checkpoint backend cannot persist the resulting
    /// state.
    pub async fn complete_pending_host_work_from_checkpoint_until_human_boundary<
        H: BpmnHostBridge,
    >(
        &self,
        request: &QianjiBpmnExecutionRequest,
        checkpoint: BpmnCheckpointEnvelope,
        completion: QianjiBpmnPendingHostCompletion,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.driver()
            .complete_pending_host_work_from_checkpoint_until_human_boundary(
                request, checkpoint, completion, host,
            )
            .await
    }

    /// Runs the BPMN request until the next host boundary or another stable
    /// outcome while reporting newly produced trace events.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the direct driver cannot create,
    /// resume, or advance the BPMN session, or when checkpoint persistence
    /// fails.
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
        self.driver()
            .run_until_host_boundary_with_trace_observer(
                request,
                host,
                resolve_initial_host_work,
                trace_observer,
            )
            .await
    }

    /// Runs until the next user/manual host boundary, resolving non-human host
    /// work through the supplied bridge.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the direct driver cannot create,
    /// resume, or advance the BPMN session, or when checkpoint persistence
    /// fails.
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
        self.driver()
            .run_until_human_boundary_with_trace_observer(
                request,
                host,
                resolve_initial_host_work,
                trace_observer,
            )
            .await
    }

    fn driver(&self) -> QianjiBpmnExecutionDriver {
        QianjiBpmnExecutionDriver::new(Arc::clone(&self.package), self.checkpoint_store.clone())
    }

    fn build_scheduler(
        &self,
    ) -> Result<Option<QianjiBpmnExecutionScheduler>, BpmnOrchestrationError> {
        if !self.should_use_scheduler_lifecycle() {
            return Ok(None);
        }

        let Some(store) = self.checkpoint_store.clone() else {
            return Ok(None);
        };
        let Some(identity) = self.scheduler_identity.as_ref() else {
            return Ok(None);
        };

        Ok(Some(
            QianjiBpmnExecutionScheduler::new(Arc::clone(&self.package), Some(store))
                .with_scheduler_identity(identity, self.scheduler_lease_ttl_ms)?,
        ))
    }

    fn should_use_scheduler_lifecycle(&self) -> bool {
        matches!(
            self.checkpoint_store,
            Some(QianjiBpmnCheckpointStore::Valkey { .. })
        ) && self
            .scheduler_identity
            .as_ref()
            .is_some_and(scheduler_identity_supports_lease)
    }
}

fn scheduler_identity_supports_lease(identity: &SchedulerAgentIdentity) -> bool {
    identity
        .agent_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}
