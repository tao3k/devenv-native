use super::routes;
use crate::bpmn::control::QianjiBpmnWorkflowControlService;
use axum::Router;
use qianji_bpmn_engine::BpmnHostBridge;

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

/// Builds an embeddable BPMN workflow-control HTTP router.
pub fn qianji_bpmn_workflow_router<H>(state: QianjiBpmnWorkflowHttpState<H>) -> Router
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    routes::router(state)
}
