//! BPMN workflow HTTP router state.
//!
//! State stays separate from the public API facade so request handlers can
//! depend on the shared router state without importing the facade owner.

use crate::bpmn::control::QianjiBpmnWorkflowControlService;
use std::sync::Arc;
use xiuxian_qianji_control::{ControlLedger, HotStateStore};

/// Shared state for the embeddable BPMN workflow HTTP router.
#[derive(Clone)]
pub struct QianjiBpmnWorkflowHttpState<H> {
    /// Lib-owned BPMN workflow control service reused by HTTP handlers.
    pub service: QianjiBpmnWorkflowControlService,
    /// Host bridge supplied by the embedding runtime.
    pub host: H,
    /// Optional durable control ledger for server-side host-work evidence.
    pub activity_evidence_ledger: Option<Arc<dyn ControlLedger>>,
    /// Optional hot-state store for explicit recovery-plan application.
    pub recovery_hot_state: Option<Arc<dyn HotStateStore>>,
}

impl<H> QianjiBpmnWorkflowHttpState<H> {
    /// Creates one HTTP router state from a workflow control service and host
    /// bridge.
    #[must_use]
    pub fn new(service: QianjiBpmnWorkflowControlService, host: H) -> Self {
        Self {
            service,
            host,
            activity_evidence_ledger: None,
            recovery_hot_state: None,
        }
    }

    /// Installs a durable control ledger for host-work activity evidence.
    #[must_use]
    pub fn with_activity_evidence_ledger(mut self, ledger: Arc<dyn ControlLedger>) -> Self {
        self.activity_evidence_ledger = Some(ledger);
        self
    }

    /// Installs a hot-state store for explicit recovery-plan application.
    #[must_use]
    pub fn with_recovery_hot_state(mut self, hot_state: Arc<dyn HotStateStore>) -> Self {
        self.recovery_hot_state = Some(hot_state);
        self
    }
}
