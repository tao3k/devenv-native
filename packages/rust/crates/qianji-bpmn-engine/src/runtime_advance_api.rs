//! Public runtime advance api contracts for BPMN/DMN engine integration.

use crate::error::Result;
use crate::host_bridge_api::BpmnHostBridge;
use crate::ir::BpmnPackage;
use crate::runtime::{BpmnInstanceState, PendingHostWork, SuspendReason, advance_instance_impl};

/// High-level outcome from one runtime advance attempt.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnAdvanceOutcome {
    /// Internal state progressed without blocking.
    Advanced,
    /// Blocked on host-dispatched work.
    BlockedOnHost(Vec<PendingHostWork>),
    /// Waiting on an external event or user/system signal.
    WaitingExternalEvent,
    /// Suspended intentionally with an optional reason.
    Suspended(Option<SuspendReason>),
    /// Completed successfully.
    Completed,
    /// Failed terminally with a message.
    Failed(String),
}

/// Advances one BPMN instance within the bounded runtime subset.
///
/// # Errors
///
/// Returns [`BpmnEngineError`] when the target process cannot be found or when
/// the current instance/model shape exceeds the supported bounded subset.
pub async fn advance_instance<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
) -> Result<BpmnAdvanceOutcome> {
    advance_instance_impl(package, instance, host).await
}
