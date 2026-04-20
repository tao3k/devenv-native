use super::adapter_error::BpmnAdapterError;
use futures::future::try_join_all;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnHostBridge, BpmnInstanceState, BpmnPackage, PendingHostWorkRequest,
    PendingHostWorkResult, advance_instance, apply_pending_host_work_result,
    build_pending_host_work_requests,
};

/// Dispatches one typed pending-host-work request through the supplied host.
///
/// # Errors
///
/// Returns [`BpmnAdapterError::Host`] when the host cannot service the request.
pub async fn dispatch_pending_host_work_request<H: BpmnHostBridge>(
    host: &H,
    request: PendingHostWorkRequest,
) -> Result<PendingHostWorkResult, BpmnAdapterError> {
    Ok(match request {
        PendingHostWorkRequest::Service(request) => {
            PendingHostWorkResult::Service(host.dispatch_service_task(request).await?)
        }
        PendingHostWorkRequest::User(request) => {
            PendingHostWorkResult::User(host.dispatch_user_task(request).await?)
        }
        PendingHostWorkRequest::Manual(request) => {
            PendingHostWorkResult::Manual(host.dispatch_manual_task(request).await?)
        }
        PendingHostWorkRequest::BusinessRule(request) => {
            PendingHostWorkResult::BusinessRule(host.dispatch_business_rule_task(request).await?)
        }
    })
}

/// Dispatches all currently materialized pending-host-work requests through the
/// supplied host, preserving request order in the returned result vector.
///
/// # Errors
///
/// Returns [`BpmnAdapterError::Host`] when any request fails at the host
/// boundary.
pub async fn dispatch_pending_host_work_requests<H: BpmnHostBridge>(
    host: &H,
    requests: Vec<PendingHostWorkRequest>,
) -> Result<Vec<PendingHostWorkResult>, BpmnAdapterError> {
    try_join_all(
        requests
            .into_iter()
            .map(|request| dispatch_pending_host_work_request(host, request)),
    )
    .await
}

/// Resolves the currently blocked pending-host-work batch, applies the
/// returned results back into the BPMN instance, and advances until the next
/// stable runtime outcome.
///
/// # Errors
///
/// Returns [`BpmnAdapterError::Engine`] when the BPMN instance is not in a
/// valid pending-host-work state or when applying host results fails.
/// Returns [`BpmnAdapterError::Host`] when the host cannot service one pending
/// request.
pub async fn resolve_pending_host_work<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
) -> Result<BpmnAdvanceOutcome, BpmnAdapterError> {
    let requests = build_pending_host_work_requests(instance)?;
    let results = dispatch_pending_host_work_requests(host, requests.clone()).await?;
    let completed_at_ms = host.now_unix_ms();

    for (request, result) in requests.into_iter().zip(results) {
        apply_pending_host_work_result(
            package,
            instance,
            request_token_id(&request),
            result,
            completed_at_ms,
        )?;
    }

    advance_instance(package, instance, host)
        .await
        .map_err(Into::into)
}

fn request_token_id(request: &PendingHostWorkRequest) -> u64 {
    match request {
        PendingHostWorkRequest::Service(request) => request.token_id,
        PendingHostWorkRequest::User(request) => request.token_id,
        PendingHostWorkRequest::Manual(request) => request.token_id,
        PendingHostWorkRequest::BusinessRule(request) => request.token_id,
    }
}
