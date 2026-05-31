//! Row and token contracts for qianji run-console projections.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Shared schema version for qianji run-console data-plane rows.
pub const QIANJI_RUN_CONSOLE_SCHEMA_VERSION: &str = "pi-wendao.qianji.run-console.v1";
/// Schema version for the server-owned durable run stream projection.
pub const QIANJI_CONTROL_RUN_STREAM_SCHEMA_VERSION: &str = "xiuxian_qianji.control.run_stream.v1";
/// Logical Flight route for control-event rows.
pub const QIANJI_RUN_CONSOLE_EVENT_ROUTE: &str = "qianji.control.run-console.events";
/// Logical Flight route for BPMN element-state rows.
pub const QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE: &str =
    "qianji.control.run-console.element-states";

/// One qianji control event projected into the run-console Arrow contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QianjiRunConsoleEventRow {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Stable event identifier inside the run-console read model.
    pub event_id: String,
    /// Ledger sequence encoded as an Arrow `Int32`.
    pub sequence: i32,
    /// Stable event-kind name.
    pub kind: String,
    /// Operator-readable event message.
    pub message: String,
    /// Optional BPMN/control step id.
    pub step_id: Option<String>,
    /// Unix timestamp in milliseconds encoded as an Arrow `Float64`.
    pub occurred_at_ms: f64,
}

/// One BPMN element state projected into the run-console Arrow contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct QianjiRunConsoleElementStateRow {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Stable BPMN element identifier.
    pub element_id: String,
    /// Server-derived runtime state.
    pub state: QianjiRunConsoleElementState,
    /// Source control event id or sequence.
    pub source_event_id: String,
    /// Operator-readable state message.
    pub message: String,
}

/// Internal projection retained so JSON HTTP and Arrow rows share derivation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QianjiRunConsoleElementProjection {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Stable BPMN element identifier.
    pub element_id: String,
    /// Server-derived runtime state for rendering markers.
    pub state: QianjiRunConsoleElementState,
    /// Control ledger sequence that produced this state.
    pub source_event_sequence: u64,
    /// Control event kind that produced this state.
    pub source_event_kind: String,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Operator-readable state message.
    pub message: String,
    /// Source metadata retained for diagnostics and hover panels.
    #[serde(default)]
    pub metadata: Value,
}

impl QianjiRunConsoleElementProjection {
    /// Convert this projection into the public element-state row contract.
    #[must_use]
    pub(crate) fn to_row(&self) -> QianjiRunConsoleElementStateRow {
        QianjiRunConsoleElementStateRow {
            run_id: self.run_id.clone(),
            element_id: self.element_id.clone(),
            state: self.state,
            source_event_id: self.source_event_sequence.to_string(),
            message: self.message.clone(),
        }
    }
}

/// Durable run-stream source lane rendered by workflow UIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QianjiControlRunStreamSource {
    /// BPMN control-plane or step lifecycle event.
    Bpmn,
    /// LLM activity event.
    Llm,
    /// Agent or subagent proposal/decision event.
    Subagent,
    /// Tool or host-work event.
    Tool,
    /// System lifecycle event.
    System,
}

impl QianjiControlRunStreamSource {
    /// Returns the stable wire value for this source lane.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bpmn => "bpmn",
            Self::Llm => "llm",
            Self::Subagent => "subagent",
            Self::Tool => "tool",
            Self::System => "system",
        }
    }
}

/// Stable control event kind label for run-stream rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QianjiControlRunStreamKind(String);

impl QianjiControlRunStreamKind {
    /// Build a kind label from a stable event-name string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Return the stable kind label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Raw DTO boundary: one row in qianji-server's durable run-stream projection.
///
/// Primitive string fields intentionally mirror the Arrow/JSON UI contract.
/// Projection code restores typed control-ledger identity before row emission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiControlRunStreamRow {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Stable row id inside the run stream.
    pub row_id: String,
    /// Ledger sequence that produced this stream row.
    pub sequence: u64,
    /// Stream lane used by UI renderers.
    pub source: QianjiControlRunStreamSource,
    /// Stable control event kind.
    pub kind: QianjiControlRunStreamKind,
    /// Compact operator-readable title.
    pub title: String,
    /// Compact operator-readable message.
    pub message: String,
    /// Activity id when the event belongs to an external activity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_id: Option<String>,
    /// Step id when the event belongs to a control step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    /// BPMN element id when the event can be pinned to the diagram.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Source metadata retained for diagnostics and richer clients.
    #[serde(default)]
    pub metadata: Value,
}

/// Server-derived marker state for one BPMN element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QianjiRunConsoleElementState {
    /// Element has pending or active work.
    Active,
    /// Element completed successfully.
    Completed,
    /// Element is failed, cancelled, or blocked.
    Failed,
}

impl QianjiRunConsoleElementState {
    /// Return the string used by the JS Arrow row contract.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}
