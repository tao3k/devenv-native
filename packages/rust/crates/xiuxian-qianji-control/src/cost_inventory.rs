//! Durable cost inventory projections from append-only event history.

use crate::{ControlEventKind, ControlEventRecord, CostObservation, RecoveryItemScope, RunId};

/// One cost inventory item derived from durable history.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CostInventoryItem {
    /// Ledger sequence that stored the observation.
    pub sequence: u64,
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Event timestamp supplied by the caller.
    pub observed_at_ms: u64,
    /// Recorded provider/tool cost.
    pub observation: CostObservation,
}

/// Replayed cost summary for budget and operator inspection.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct CostInventorySummary {
    /// Total observations included by the projection.
    pub total: usize,
    /// Run-scoped observations.
    pub run_scoped: usize,
    /// Step-scoped observations.
    pub step_scoped: usize,
    /// Total observed tokens.
    pub total_tokens: u64,
    /// Total observed cost in USD micros.
    pub cost_usd_micros: u64,
    /// Total observed latency for rows that report latency.
    pub latency_ms: u64,
    /// Number of observations that report latency.
    pub latency_observations: usize,
}

impl CostInventorySummary {
    fn record(&mut self, scope: &RecoveryItemScope, observation: &CostObservation) {
        self.total += 1;
        match scope {
            RecoveryItemScope::Run => self.run_scoped += 1,
            RecoveryItemScope::Step { .. } => self.step_scoped += 1,
        }
        self.total_tokens += observation.observed_total_tokens();
        self.cost_usd_micros += observation.cost_usd_micros;
        if let Some(latency_ms) = observation.latency_ms {
            self.latency_ms += latency_ms;
            self.latency_observations += 1;
        }
    }
}

/// Read-only durable cost projection for run and step budget inspection.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CostInventoryProjection {
    /// Owning run id.
    pub run_id: RunId,
    /// Cost observations replayed from run and step scopes.
    #[serde(default)]
    pub items: Vec<CostInventoryItem>,
    /// Scope and aggregate counts for all included observations.
    #[serde(default)]
    pub summary: CostInventorySummary,
}

impl CostInventoryProjection {
    /// Projects durable cost observations from event records.
    ///
    /// Non-cost events are ignored. The original event sequence and timestamp
    /// are retained so operators can audit budget chronology.
    #[must_use]
    pub fn from_records(run_id: RunId, records: &[ControlEventRecord]) -> Self {
        let mut items = Vec::new();
        let mut summary = CostInventorySummary::default();

        for record in records {
            let ControlEventKind::CostObserved { observation } = &record.event.kind else {
                continue;
            };
            let scope = record
                .event
                .step_id
                .clone()
                .map_or_else(RecoveryItemScope::run, RecoveryItemScope::step);
            summary.record(&scope, observation);
            items.push(CostInventoryItem {
                sequence: record.sequence,
                scope,
                observed_at_ms: record.event.occurred_at_ms,
                observation: observation.clone(),
            });
        }

        Self {
            run_id,
            items,
            summary,
        }
    }
}
