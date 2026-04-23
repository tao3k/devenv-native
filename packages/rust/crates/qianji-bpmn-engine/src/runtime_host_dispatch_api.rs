//! Public runtime host-dispatch entrypoints.

use crate::error::Result;
use crate::host_types_api::PendingHostWorkRequest;
use crate::runtime::BpmnInstanceState;

/// Builds a typed host-dispatch request from the currently blocked BPMN
/// instance state.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingPendingHostWork`] when the instance is not
/// currently blocked on host work, or
/// [`BpmnEngineError::AmbiguousPendingHostWork`] when more than one pending
/// host-work entry exists.
pub fn build_pending_host_work_request(
    instance: &BpmnInstanceState,
) -> Result<PendingHostWorkRequest> {
    crate::runtime::build_pending_host_work_request(instance)
}

/// Builds typed host-dispatch requests from every currently blocked BPMN token.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingPendingHostWork`] when the instance is not
/// currently blocked on host work.
pub fn build_pending_host_work_requests(
    instance: &BpmnInstanceState,
) -> Result<Vec<PendingHostWorkRequest>> {
    crate::runtime::build_pending_host_work_requests(instance)
}
