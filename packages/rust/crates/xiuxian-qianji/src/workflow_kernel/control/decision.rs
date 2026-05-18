use xiuxian_qianji_control::{
    ControlError, ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    CostObservation, EvidenceRef, GateResult, RecoveryAttempt, RunId, RunView, StepId,
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
    /// Control-plane step id derived from the stage id.
    pub step_id: StepId,
    /// Number of events appended by this recording call.
    pub appended_event_count: usize,
    /// Ledger records returned by append operations.
    pub records: Vec<ControlEventRecord>,
    /// Ledger-replayed view after recording completed.
    pub run_view: RunView,
}

/// Records managed workflow stage decision facts into an injected control ledger.
///
/// Events are appended in deterministic evidence, gate-result, then cost order.
///
/// # Errors
///
/// Returns a control error when the workflow id or stage id is blank, when the
/// decision contains no facts, or when the ledger rejects an event append or
/// replay.
pub fn record_workflow_stage_decision(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    decision: WorkflowStageDecisionRecord,
) -> ControlResult<WorkflowStageDecisionRecordingOutcome> {
    let run_id = RunId::new(workflow_id.to_owned())?;
    let step_id = StepId::new(stage_id.to_owned())?;
    if decision.is_empty() {
        return Err(ControlError::InvalidEventSequence {
            message: "workflow stage decision contains no control facts".to_owned(),
        });
    }

    let mut events = Vec::with_capacity(
        decision.evidence.len() + decision.gate_results.len() + decision.cost_observations.len(),
    );
    events.extend(decision.evidence.into_iter().map(|evidence| {
        ControlEvent::step(
            run_id.clone(),
            step_id.clone(),
            occurred_at_ms,
            ControlEventKind::EvidenceAttached { evidence },
        )
    }));
    events.extend(decision.gate_results.into_iter().map(|result| {
        ControlEvent::step(
            run_id.clone(),
            step_id.clone(),
            occurred_at_ms,
            ControlEventKind::GateEvaluated { result },
        )
    }));
    events.extend(decision.cost_observations.into_iter().map(|observation| {
        ControlEvent::step(
            run_id.clone(),
            step_id.clone(),
            occurred_at_ms,
            ControlEventKind::CostObserved { observation },
        )
    }));

    append_events_and_replay(ledger, run_id, step_id, events)
}

/// Records a gate-driven workflow stage recovery decision into a control ledger.
///
/// Events are appended in deterministic evidence, gate-result, cost, then
/// recovery order. The decision must include at least one failed gate result.
///
/// # Errors
///
/// Returns a control error when the workflow id or stage id is blank, when the
/// decision contains no failed gate result, or when the ledger rejects an event
/// append or replay.
pub fn record_workflow_stage_recovery_decision(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    recovery: WorkflowStageRecoveryDecisionRecord,
) -> ControlResult<WorkflowStageDecisionRecordingOutcome> {
    let run_id = RunId::new(workflow_id.to_owned())?;
    let step_id = StepId::new(stage_id.to_owned())?;
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
        ControlEvent::step(
            run_id.clone(),
            step_id.clone(),
            occurred_at_ms,
            ControlEventKind::EvidenceAttached { evidence },
        )
    }));
    events.extend(recovery.decision.gate_results.into_iter().map(|result| {
        ControlEvent::step(
            run_id.clone(),
            step_id.clone(),
            occurred_at_ms,
            ControlEventKind::GateEvaluated { result },
        )
    }));
    events.extend(
        recovery
            .decision
            .cost_observations
            .into_iter()
            .map(|observation| {
                ControlEvent::step(
                    run_id.clone(),
                    step_id.clone(),
                    occurred_at_ms,
                    ControlEventKind::CostObserved { observation },
                )
            }),
    );
    events.push(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        occurred_at_ms,
        ControlEventKind::RecoveryStarted {
            attempt: recovery.recovery_attempt,
        },
    ));

    append_events_and_replay(ledger, run_id, step_id, events)
}

fn append_events_and_replay(
    ledger: &dyn ControlLedger,
    run_id: RunId,
    step_id: StepId,
    events: Vec<ControlEvent>,
) -> ControlResult<WorkflowStageDecisionRecordingOutcome> {
    let records = events
        .into_iter()
        .map(|event| ledger.append_event(event))
        .collect::<ControlResult<Vec<_>>>()?;
    let run_view = ledger.load_run_view(&run_id)?;
    Ok(WorkflowStageDecisionRecordingOutcome {
        run_id,
        step_id,
        appended_event_count: records.len(),
        records,
        run_view,
    })
}
