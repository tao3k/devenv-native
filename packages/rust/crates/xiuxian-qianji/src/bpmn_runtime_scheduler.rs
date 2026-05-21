//! Bpmn runtime scheduler surface for `xiuxian-qianji`.

use super::backend::QianjiBpmnCheckpointStore;
use super::driver::{QianjiBpmnExecutionDriver, QianjiBpmnExecutionReport};
use super::error::BpmnOrchestrationError;
use super::ownership::QianjiBpmnSchedulerLeaseConfig;
use super::session::QianjiBpmnSession;
use crate::scheduler_identity::SchedulerAgentIdentity;
use qianji_bpmn_engine::{BpmnExecutionTraceEvent, BpmnHostBridge, BpmnPackage};
use std::sync::Arc;

use super::driver::QianjiBpmnExecutionRequest;

/// BPMN-specific execution scheduler above the shared execution driver.
///
/// This scheduler keeps BPMN semantics in `qianji-bpmn-engine` while applying
/// scheduler-style checkpoint lifecycle rules in the host crate: waiting and
/// suspended runs remain resumable, while terminal runs clean up stored
/// checkpoints.
#[derive(Debug, Clone)]
pub struct QianjiBpmnExecutionScheduler {
    driver: QianjiBpmnExecutionDriver,
    checkpoint_lease: Option<QianjiBpmnSchedulerLeaseConfig>,
}

impl QianjiBpmnExecutionScheduler {
    /// Creates one BPMN-specific execution scheduler from a loaded package plus
    /// optional checkpoint storage.
    #[must_use]
    pub fn new(
        package: Arc<BpmnPackage>,
        checkpoint_store: Option<QianjiBpmnCheckpointStore>,
    ) -> Self {
        Self {
            driver: QianjiBpmnExecutionDriver::new(package, checkpoint_store),
            checkpoint_lease: None,
        }
    }

    /// Configures one Valkey-backed lease owner for scheduler-managed BPMN
    /// checkpoint lifecycle.
    #[must_use]
    pub fn with_checkpoint_lease(
        mut self,
        checkpoint_lease: QianjiBpmnSchedulerLeaseConfig,
    ) -> Self {
        self.checkpoint_lease = Some(checkpoint_lease);
        self
    }

    /// Configures one Valkey-backed lease owner from one scheduler execution
    /// identity.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the scheduler identity does not
    /// expose a stable `agent_id` for single-writer lease ownership.
    pub fn with_scheduler_identity(
        mut self,
        scheduler_identity: &SchedulerAgentIdentity,
        lease_ttl_ms: u64,
    ) -> Result<Self, BpmnOrchestrationError> {
        self.checkpoint_lease = Some(QianjiBpmnSchedulerLeaseConfig::from_scheduler_identity(
            scheduler_identity,
            lease_ttl_ms,
        )?);
        Ok(self)
    }

    /// Runs the BPMN session until the next stable outcome with scheduler-style
    /// checkpoint lifecycle ownership.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when session creation/resume fails,
    /// when the host cannot service BPMN work, or when checkpoint persistence
    /// or cleanup fails.
    pub async fn run<H: BpmnHostBridge>(
        &self,
        request: &QianjiBpmnExecutionRequest,
        host: &H,
    ) -> Result<QianjiBpmnExecutionReport, BpmnOrchestrationError> {
        self.driver
            .run_with_scheduler_lifecycle(request, host, self.checkpoint_lease.as_ref())
            .await
    }

    /// Runs the BPMN session with scheduler-style checkpoint lifecycle while
    /// reporting newly produced trace events after each runtime step.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when session creation/resume fails,
    /// when the host cannot service BPMN work, or when checkpoint persistence
    /// or cleanup fails.
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
        self.driver
            .run_with_scheduler_lifecycle_and_trace_observer(
                request,
                host,
                self.checkpoint_lease.as_ref(),
                trace_observer,
            )
            .await
    }
}
