//! Server-owned BPMN execution-graph projection for control runs.

use crate::bpmn::run_console_read_model::{
    QianjiRunConsoleElementProjection, QianjiRunConsoleElementState,
    qianji_run_console_element_projections,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use xiuxian_qianji_control::{ControlEventRecord, RunId};

/// HTTP response for one control-ledger execution-graph query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct QianjiControlExecutionGraphHttpResponse {
    /// Stable control-plane run identifier.
    pub(super) run_id: String,
    /// Number of BPMN elements projected from the control run.
    pub(super) element_count: usize,
    /// Stable BPMN element states keyed by server-recorded element ids.
    #[serde(default)]
    pub(super) elements: Vec<QianjiControlExecutionGraphElementHttpResponse>,
}

impl QianjiControlExecutionGraphHttpResponse {
    pub(super) fn from_events(run_id: &RunId, events: &[ControlEventRecord]) -> Self {
        let elements = qianji_run_console_element_projections(run_id, events)
            .into_iter()
            .map(QianjiControlExecutionGraphElementHttpResponse::from)
            .collect::<Vec<_>>();
        Self {
            run_id: run_id.as_str().to_owned(),
            element_count: elements.len(),
            elements,
        }
    }
}

/// Server-normalized state for one BPMN element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct QianjiControlExecutionGraphElementHttpResponse {
    /// Stable BPMN element identifier.
    pub(super) element_id: String,
    /// Server-derived runtime state for rendering markers.
    pub(super) state: QianjiRunConsoleElementState,
    /// Control ledger sequence that produced this state.
    pub(super) source_event_sequence: u64,
    /// Control event kind that produced this state.
    pub(super) source_event_kind: String,
    /// Event timestamp in Unix milliseconds.
    pub(super) occurred_at_ms: u64,
    /// Operator-readable state message.
    pub(super) message: String,
    /// Source metadata retained for diagnostics and hover panels.
    #[serde(default)]
    pub(super) metadata: Value,
}

impl From<QianjiRunConsoleElementProjection> for QianjiControlExecutionGraphElementHttpResponse {
    fn from(value: QianjiRunConsoleElementProjection) -> Self {
        Self {
            element_id: value.element_id,
            state: value.state,
            source_event_sequence: value.source_event_sequence,
            source_event_kind: value.source_event_kind,
            occurred_at_ms: value.occurred_at_ms,
            message: value.message,
            metadata: value.metadata,
        }
    }
}
