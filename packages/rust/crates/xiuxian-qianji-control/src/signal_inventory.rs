//! Durable signal inventory projections from append-only event history.

use crate::{ControlEventKind, ControlEventRecord, RecoveryItemScope, RunId, SignalRecord};

/// One signal inventory item derived from durable history.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignalInventoryItem {
    /// Ledger sequence that stored the signal.
    pub sequence: u64,
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Event timestamp supplied by the caller.
    pub received_at_ms: u64,
    /// Recorded signal payload.
    pub signal: SignalRecord,
}

/// Replayed signal counts for HITL and external-event operators.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct SignalInventorySummary {
    /// Total signals included by the projection.
    pub total: usize,
    /// Run-scoped signals.
    pub run_scoped: usize,
    /// Step-scoped signals.
    pub step_scoped: usize,
}

impl SignalInventorySummary {
    fn record(&mut self, scope: &RecoveryItemScope) {
        self.total += 1;
        match scope {
            RecoveryItemScope::Run => self.run_scoped += 1,
            RecoveryItemScope::Step { .. } => self.step_scoped += 1,
        }
    }
}

/// Read-only durable signal projection for run and step external-event
/// inspection.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignalInventoryProjection {
    /// Owning run id.
    pub run_id: RunId,
    /// Signals replayed from run and step scopes.
    #[serde(default)]
    pub items: Vec<SignalInventoryItem>,
    /// Scope counts for all included signals.
    #[serde(default)]
    pub summary: SignalInventorySummary,
}

impl SignalInventoryProjection {
    /// Projects durable signals from event records.
    ///
    /// Non-signal events are ignored. The original event sequence and
    /// timestamp are retained so operators can audit external input ordering.
    #[must_use]
    pub fn from_records(run_id: RunId, records: &[ControlEventRecord]) -> Self {
        let mut items = Vec::new();
        let mut summary = SignalInventorySummary::default();

        for record in records {
            let ControlEventKind::SignalReceived { signal } = &record.event.kind else {
                continue;
            };
            let scope = record
                .event
                .step_id
                .clone()
                .map_or_else(RecoveryItemScope::run, RecoveryItemScope::step);
            summary.record(&scope);
            items.push(SignalInventoryItem {
                sequence: record.sequence,
                scope,
                received_at_ms: record.event.occurred_at_ms,
                signal: signal.clone(),
            });
        }

        Self {
            run_id,
            items,
            summary,
        }
    }
}
