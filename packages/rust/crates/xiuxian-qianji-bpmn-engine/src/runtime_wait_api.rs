//! Public runtime wait state and event-poll entrypoints.

use crate::error::Result;
use crate::host_types_api::{EventPollOutcome, EventPollRequest};
use crate::ir::BpmnPackage;
use crate::ir_event_api::BpmnEventKind;
use crate::ir_event_api::BpmnTimerSpec;
use crate::ir_index_api::BpmnNodeIndex;
use crate::runtime::BpmnInstanceState;
use crate::runtime_advance_api::BpmnAdvanceOutcome;
use std::borrow::Borrow;

/// Waiting categories that require external progress.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitKind {
    /// Waiting on an external event.
    ExternalEvent,
    /// Waiting on user action.
    UserAction,
    /// Waiting on a timer or wall-clock boundary.
    Timer,
    /// Waiting on a bounded BPMN conditional expression.
    Conditional,
}

/// Waiting registration for one node.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WaitRegistration {
    /// Optional owning process identifier for waits that belong to a
    /// suspended parent frame instead of the currently active process.
    #[serde(default)]
    pub process_id: Option<String>,
    /// Owning node index.
    pub node_index: BpmnNodeIndex,
    /// Optional currently blocked host-work node for boundary waits.
    pub blocking_node_index: Option<BpmnNodeIndex>,
    /// Wait category.
    pub kind: WaitKind,
    /// Optional BPMN event kind when the wait originates from an event node.
    pub event_kind: Option<BpmnEventKind>,
    /// Optional source-level event reference such as `messageRef`.
    pub event_reference: Option<String>,
    /// Optional resolved event name or label.
    pub event_name: Option<String>,
    /// Optional timer-definition snapshot for timer waits.
    pub timer: Option<BpmnTimerSpec>,
    /// Optional bounded condition expression for conditional-event waits.
    #[serde(default)]
    pub condition_expression: Option<String>,
    /// Optional host-level deduplication key derived from the explicit event
    /// reference. This is not BPMN correlation matching.
    pub deduplication_key: Option<String>,
}

/// Builds one typed event-poll request from the current blocked wait state.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingWaitRegistration`] when the instance does
/// not currently hold a wait registration, or [`BpmnEngineError`] when the
/// current wait shape exceeds the bounded single-wait slice.
pub fn build_event_poll_request(instance: &BpmnInstanceState) -> Result<EventPollRequest> {
    crate::runtime::build_event_poll_request(instance)
}

/// Applies one external-event poll outcome to the currently blocked instance.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingWaitRegistration`] when the instance does
/// not currently hold a wait registration, or [`BpmnEngineError`] when the
/// current wait/runtime shape exceeds the bounded single-wait slice.
pub fn apply_event_poll_outcome(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    outcome: impl Borrow<EventPollOutcome>,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    crate::runtime::apply_event_poll_outcome(package, instance, outcome, polled_at_ms)
}
