//! Workflow-neutral helpers for recording managed step decisions.

use crate::{
    ControlError, ControlEventRecord, ControlLedger, ControlResult, CostObservation,
    CostObservationJournalRecord, EvidenceRef, GateResult, RecoveryAttempt, RecoveryItemScope,
    RecoveryStartedJournalRecord, RunId, RunView, StepEvidenceJournalRecord,
    StepGateResultJournalRecord, StepId, record_control_event_batch,
};

/// Caller-supplied decision facts for one workflow stage.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorkflowStageDecisionRecord {
    /// Evidence references that justify the decision.
    pub evidence: Vec<EvidenceRef>,
    /// Gate results evaluated by the caller.
    pub gate_results: Vec<GateResult>,
    /// Cost observations associated with the decision.
    pub cost_observations: Vec<CostObservation>,
}

impl WorkflowStageDecisionRecord {
    /// Returns true when the record has no facts to append.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.evidence.is_empty()
            && self.gate_results.is_empty()
            && self.cost_observations.is_empty()
    }
}

/// Caller-supplied gate-driven recovery decision for one workflow stage.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowStageRecoveryDecisionRecord {
    /// Decision facts that justify the recovery.
    pub decision: WorkflowStageDecisionRecord,
    /// Recovery attempt to append after decision facts.
    pub recovery_attempt: RecoveryAttempt,
}

/// Summary returned after recording managed decision facts for one stage.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowStageDecisionRecordingOutcome {
    /// Control-plane run id derived from the workflow id.
    pub run_id: RunId,
    /// Control-plane step id derived from the workflow stage.
    pub step_id: StepId,
    /// Number of events appended by this recording call.
    pub appended_event_count: usize,
    /// Ledger records returned by append operations.
    pub records: Vec<ControlEventRecord>,
    /// Ledger-replayed view after recording completed.
    pub run_view: RunView,
}

/// Request for recording stage-level workflow decision facts.
pub struct WorkflowStageDecisionRecordingRequest<'ledger> {
    /// Ledger that owns the appended control events.
    pub ledger: &'ledger dyn ControlLedger,
    /// Control-plane run id for the workflow.
    pub run_id: RunId,
    /// Control-plane step id for the workflow stage.
    pub step_id: StepId,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Decision facts to append.
    pub decision: WorkflowStageDecisionRecord,
}

/// Request for recording a gate-driven workflow stage recovery decision.
pub struct WorkflowStageRecoveryDecisionRecordingRequest<'ledger> {
    /// Ledger that owns the appended control events.
    pub ledger: &'ledger dyn ControlLedger,
    /// Control-plane run id for the workflow.
    pub run_id: RunId,
    /// Control-plane step id for the workflow stage.
    pub step_id: StepId,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Recovery decision facts to append.
    pub recovery: WorkflowStageRecoveryDecisionRecord,
}

/// Records managed workflow stage decision facts into an injected control ledger.
///
/// Events are appended in deterministic evidence, gate-result, then cost order.
///
/// # Errors
///
/// Returns a control error when the workflow id or step id is blank, when the
/// decision contains no facts, or when the ledger rejects an event append or
/// replay.
pub fn record_workflow_stage_decision(
    request: WorkflowStageDecisionRecordingRequest<'_>,
) -> ControlResult<WorkflowStageDecisionRecordingOutcome> {
    let WorkflowStageDecisionRecordingRequest {
        ledger,
        run_id,
        step_id,
        occurred_at_ms,
        decision,
    } = request;
    if decision.is_empty() {
        return Err(ControlError::InvalidEventSequence {
            message: "workflow stage decision contains no control facts".to_owned(),
        });
    }

    let mut events = Vec::with_capacity(
        decision.evidence.len() + decision.gate_results.len() + decision.cost_observations.len(),
    );
    events.extend(decision.evidence.into_iter().map(|evidence| {
        StepEvidenceJournalRecord::new(run_id.clone(), step_id.clone(), evidence, occurred_at_ms)
            .into_event()
    }));
    events.extend(decision.gate_results.into_iter().map(|result| {
        StepGateResultJournalRecord::new(run_id.clone(), step_id.clone(), result, occurred_at_ms)
            .into_event()
    }));
    events.extend(decision.cost_observations.into_iter().map(|observation| {
        CostObservationJournalRecord::step(
            run_id.clone(),
            step_id.clone(),
            observation,
            occurred_at_ms,
        )
        .into_event()
    }));

    record_decision_event_batch(ledger, step_id, events)
}

/// Records a gate-driven workflow stage recovery decision into a control ledger.
///
/// Events are appended in deterministic evidence, gate-result, cost, then
/// recovery order. The decision must include at least one failed gate result.
///
/// # Errors
///
/// Returns a control error when the workflow id or step id is blank, when the
/// decision contains no failed gate result, or when the ledger rejects an event
/// append or replay.
pub fn record_workflow_stage_recovery_decision(
    request: WorkflowStageRecoveryDecisionRecordingRequest<'_>,
) -> ControlResult<WorkflowStageDecisionRecordingOutcome> {
    let WorkflowStageRecoveryDecisionRecordingRequest {
        ledger,
        run_id,
        step_id,
        occurred_at_ms,
        recovery,
    } = request;
    if !recovery
        .decision
        .gate_results
        .iter()
        .any(|result| !result.passed)
    {
        return Err(ControlError::InvalidEventSequence {
            message: "workflow stage recovery decision requires a failed gate result".to_owned(),
        });
    }

    let mut events = Vec::with_capacity(
        recovery.decision.evidence.len()
            + recovery.decision.gate_results.len()
            + recovery.decision.cost_observations.len()
            + 1,
    );
    events.extend(recovery.decision.evidence.into_iter().map(|evidence| {
        StepEvidenceJournalRecord::new(run_id.clone(), step_id.clone(), evidence, occurred_at_ms)
            .into_event()
    }));
    events.extend(recovery.decision.gate_results.into_iter().map(|result| {
        StepGateResultJournalRecord::new(run_id.clone(), step_id.clone(), result, occurred_at_ms)
            .into_event()
    }));
    events.extend(
        recovery
            .decision
            .cost_observations
            .into_iter()
            .map(|observation| {
                CostObservationJournalRecord::step(
                    run_id.clone(),
                    step_id.clone(),
                    observation,
                    occurred_at_ms,
                )
                .into_event()
            }),
    );
    events.push(
        RecoveryStartedJournalRecord::new(
            run_id.clone(),
            RecoveryItemScope::Step {
                step_id: step_id.clone(),
            },
            recovery.recovery_attempt,
            occurred_at_ms,
        )
        .into_event(),
    );

    record_decision_event_batch(ledger, step_id, events)
}

fn record_decision_event_batch(
    ledger: &dyn ControlLedger,
    step_id: StepId,
    events: Vec<crate::ControlEvent>,
) -> ControlResult<WorkflowStageDecisionRecordingOutcome> {
    let outcome = record_control_event_batch(ledger, events)?;
    Ok(WorkflowStageDecisionRecordingOutcome {
        run_id: outcome.run_id,
        step_id,
        appended_event_count: outcome.appended_event_count,
        records: outcome.records,
        run_view: outcome.run_view,
    })
}
