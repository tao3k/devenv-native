//! Evidence, gate, and cost observation journal recording helpers.

use crate::{
    ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    CostObservation, EvidenceRef, GateResult, RecoveryItemScope, RunId, StepId,
};

/// Named request for recording one step evidence fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StepEvidenceJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Owning step id.
    pub step_id: StepId,
    /// Evidence reference to attach.
    pub evidence: EvidenceRef,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl StepEvidenceJournalRecord {
    /// Creates a step evidence journal record request.
    #[must_use]
    pub const fn new(
        run_id: RunId,
        step_id: StepId,
        evidence: EvidenceRef,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            step_id,
            evidence,
            occurred_at_ms,
        }
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        ControlEvent::step(
            self.run_id,
            self.step_id,
            self.occurred_at_ms,
            ControlEventKind::EvidenceAttached {
                evidence: self.evidence,
            },
        )
    }
}

/// Records one step evidence fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_step_evidence<L>(
    ledger: &L,
    request: StepEvidenceJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}

/// Named request for recording one gate result fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StepGateResultJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Owning step id.
    pub step_id: StepId,
    /// Gate result to attach.
    pub result: GateResult,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl StepGateResultJournalRecord {
    /// Creates a step gate-result journal record request.
    #[must_use]
    pub const fn new(
        run_id: RunId,
        step_id: StepId,
        result: GateResult,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            step_id,
            result,
            occurred_at_ms,
        }
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        ControlEvent::step(
            self.run_id,
            self.step_id,
            self.occurred_at_ms,
            ControlEventKind::GateEvaluated {
                result: self.result,
            },
        )
    }
}

/// Records one step gate-result fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_step_gate_result<L>(
    ledger: &L,
    request: StepGateResultJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}

/// Named request for recording one cost observation fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CostObservationJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Run or step scope for the cost observation.
    pub scope: RecoveryItemScope,
    /// Cost observation to attach.
    pub observation: CostObservation,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl CostObservationJournalRecord {
    /// Creates a run-scoped cost observation journal record request.
    #[must_use]
    pub const fn run(run_id: RunId, observation: CostObservation, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            scope: RecoveryItemScope::Run,
            observation,
            occurred_at_ms,
        }
    }

    /// Creates a step-scoped cost observation journal record request.
    #[must_use]
    pub const fn step(
        run_id: RunId,
        step_id: StepId,
        observation: CostObservation,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            scope: RecoveryItemScope::Step { step_id },
            observation,
            occurred_at_ms,
        }
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let kind = ControlEventKind::CostObserved {
            observation: self.observation,
        };
        match self.scope {
            RecoveryItemScope::Run => ControlEvent::run(self.run_id, self.occurred_at_ms, kind),
            RecoveryItemScope::Step { step_id } => {
                ControlEvent::step(self.run_id, step_id, self.occurred_at_ms, kind)
            }
        }
    }
}

/// Records one cost observation fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_cost_observation<L>(
    ledger: &L,
    request: CostObservationJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}
