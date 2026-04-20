use super::adapter_error::BpmnAdapterError;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnHostBridge, BpmnInstanceState, BpmnPackage, HostBridgeError,
    apply_event_poll_outcome, build_event_poll_request,
};

/// Polls one waiting BPMN instance through the supplied host bridge and
/// applies the resulting external-event outcome when available.
///
/// If the host leaves `poll_external_event(...)` unsupported, the instance
/// remains stably waiting and this helper returns
/// [`BpmnAdvanceOutcome::WaitingExternalEvent`] instead of surfacing a hard
/// adapter error. This preserves the current default behavior for hosts that
/// have not implemented external-event delivery yet.
///
/// # Errors
///
/// Returns [`BpmnAdapterError::Engine`] when the BPMN instance is not in a
/// valid waiting state or when applying the host outcome fails.
/// Returns [`BpmnAdapterError::Host`] when the host reports a non-unsupported
/// event-poll failure.
pub async fn resolve_waiting_external_event<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
) -> Result<BpmnAdvanceOutcome, BpmnAdapterError> {
    let request = build_event_poll_request(instance)?;
    let outcome = match host.poll_external_event(request).await {
        Ok(outcome) => outcome,
        Err(HostBridgeError::UnsupportedOperation {
            operation: "poll_external_event",
        }) => {
            return Ok(BpmnAdvanceOutcome::WaitingExternalEvent);
        }
        Err(error) => return Err(error.into()),
    };

    apply_event_poll_outcome(package, instance, outcome, host.now_unix_ms()).map_err(Into::into)
}
