//! `query_core::telemetry` owns Wendao query core telemetry behavior.

use std::sync::{Arc, Mutex};

use crate::query_core::types::{WendaoBackendKind, WendaoOperatorKind};

/// Single explain event emitted by Phase-1 query-core execution.
#[derive(Debug, Clone)]
pub struct WendaoExplainEvent {
    /// Operator that emitted the event.
    pub operator_kind: WendaoOperatorKind,
    /// Backend that executed the work.
    pub backend_kind: WendaoBackendKind,
    /// Whether execution delegated to a legacy adapter.
    pub legacy_adapter: bool,
    /// Input row count when known.
    pub input_row_count: Option<usize>,
    /// Output row count when known.
    pub output_row_count: Option<usize>,
    /// Whether this event corresponds to payload hydration.
    pub payload_fetch: bool,
    /// Surviving row count after narrow-phase filtering.
    pub narrow_phase_surviving_count: Option<usize>,
    /// Row count fetched during payload hydration.
    pub payload_phase_fetched_count: Option<usize>,
    /// Optional human-readable note.
    pub note: Option<String>,
}

/// Sink for query-core explain events.
pub trait WendaoExplainSink: Send + Sync {
    /// Record a single explain event.
    fn record(&self, event: WendaoExplainEvent);
}

/// Sink that discards explain events.
#[derive(Default)]
pub struct NoopWendaoExplainSink;

impl WendaoExplainSink for NoopWendaoExplainSink {
    fn record(&self, _event: WendaoExplainEvent) {}
}

/// In-memory explain sink used by tests and early adopters.
#[derive(Default, Clone)]
pub struct InMemoryWendaoExplainSink {
    events: Arc<Mutex<Vec<WendaoExplainEvent>>>,
}

impl InMemoryWendaoExplainSink {
    /// Create an empty in-memory explain sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot all recorded explain events.
    #[must_use]
    pub fn events(&self) -> Vec<WendaoExplainEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl WendaoExplainSink for InMemoryWendaoExplainSink {
    fn record(&self, event: WendaoExplainEvent) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
    }
}

/// Format a compact summary of query-core explain events for internal logs.
#[must_use]
pub fn explain_events_summary(events: &[WendaoExplainEvent]) -> String {
    if events.is_empty() {
        return "events=0".to_string();
    }

    events
        .iter()
        .map(|event| {
            let input = event
                .input_row_count
                .map_or_else(|| "?".to_string(), |count| count.to_string());
            let output = event
                .output_row_count
                .map_or_else(|| "?".to_string(), |count| count.to_string());
            format!(
                "operator={:?} backend={:?} legacy={} rows={input}->{output} payload={} narrow={:?} fetched={:?}",
                event.operator_kind,
                event.backend_kind,
                event.legacy_adapter,
                event.payload_fetch,
                event.narrow_phase_surviving_count,
                event.payload_phase_fetched_count,
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}
