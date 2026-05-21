//! BPMN workflow HTTP router state.
//!
//! State stays separate from the public API facade so request handlers can
//! depend on the shared router state without importing the facade owner.

use crate::bpmn::control::QianjiBpmnWorkflowControlService;

/// Shared state for the embeddable BPMN workflow HTTP router.
#[derive(Clone)]
pub struct QianjiBpmnWorkflowHttpState<H> {
    /// Lib-owned BPMN workflow control service reused by HTTP handlers.
    pub service: QianjiBpmnWorkflowControlService,
    /// Host bridge supplied by the embedding runtime.
    pub host: H,
}

impl<H> QianjiBpmnWorkflowHttpState<H> {
    /// Creates one HTTP router state from a workflow control service and host
    /// bridge.
    #[must_use]
    pub fn new(service: QianjiBpmnWorkflowControlService, host: H) -> Self {
        Self { service, host }
    }
}
