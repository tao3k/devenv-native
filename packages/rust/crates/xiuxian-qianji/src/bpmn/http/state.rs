//! BPMN workflow HTTP router state.
//!
//! State stays separate from the public API facade so request handlers can
//! depend on the shared router state without importing the facade owner.

use crate::bpmn::control::QianjiBpmnWorkflowControlService;
use std::sync::Arc;
use xiuxian_qianji_control::ControlLedger;

/// Shared state for the embeddable BPMN workflow HTTP router.
#[derive(Clone)]
pub struct QianjiBpmnWorkflowHttpState<H> {
    /// Lib-owned BPMN workflow control service reused by HTTP handlers.
    pub service: QianjiBpmnWorkflowControlService,
    /// Host bridge supplied by the embedding runtime.
    pub host: H,
    /// Optional durable control ledger for server-side host-work evidence.
    pub activity_evidence_ledger: Option<Arc<dyn ControlLedger>>,
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
        }
    }

    /// Installs a durable control ledger for host-work activity evidence.
    #[must_use]
    pub fn with_activity_evidence_ledger(mut self, ledger: Arc<dyn ControlLedger>) -> Self {
        self.activity_evidence_ledger = Some(ledger);
        self
    }
}
