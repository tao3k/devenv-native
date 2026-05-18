use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    CostObservation, EvidenceRef, GateResult, RecoveryAttempt, RunId, StepId,
};

/// Records a workflow run recovery attempt into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id is blank, or when the ledger
/// rejects the recovery event append.
pub fn record_workflow_run_recovery_attempt(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    occurred_at_ms: u64,
    attempt: RecoveryAttempt,
) -> ControlResult<ControlEventRecord> {
    let run_id = RunId::new(workflow_id.to_owned())?;
    ledger.append_event(ControlEvent::run(
        run_id,
        occurred_at_ms,
        ControlEventKind::RecoveryStarted { attempt },
    ))
}

/// Records a workflow run cost observation into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id is blank, or when the ledger
/// rejects the cost event append.
pub fn record_workflow_run_cost_observation(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    occurred_at_ms: u64,
    observation: CostObservation,
) -> ControlResult<ControlEventRecord> {
    let run_id = RunId::new(workflow_id.to_owned())?;
    ledger.append_event(ControlEvent::run(
        run_id,
        occurred_at_ms,
        ControlEventKind::CostObserved { observation },
    ))
}

/// Records a workflow stage recovery attempt into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id or stage id is blank, or when
/// the ledger rejects the recovery event append.
pub fn record_workflow_stage_recovery_attempt(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    attempt: RecoveryAttempt,
) -> ControlResult<ControlEventRecord> {
    let run_id = RunId::new(workflow_id.to_owned())?;
    let step_id = StepId::new(stage_id.to_owned())?;
    ledger.append_event(ControlEvent::step(
        run_id,
        step_id,
        occurred_at_ms,
        ControlEventKind::RecoveryStarted { attempt },
    ))
}

/// Records workflow stage evidence into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id or stage id is blank, or when
/// the ledger rejects the evidence event append.
pub fn record_workflow_stage_evidence(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    evidence: EvidenceRef,
) -> ControlResult<ControlEventRecord> {
    let run_id = RunId::new(workflow_id.to_owned())?;
    let step_id = StepId::new(stage_id.to_owned())?;
    ledger.append_event(ControlEvent::step(
        run_id,
        step_id,
        occurred_at_ms,
        ControlEventKind::EvidenceAttached { evidence },
    ))
}

/// Records a workflow stage cost observation into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id or stage id is blank, or when
/// the ledger rejects the cost event append.
pub fn record_workflow_stage_cost_observation(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    observation: CostObservation,
) -> ControlResult<ControlEventRecord> {
    let run_id = RunId::new(workflow_id.to_owned())?;
    let step_id = StepId::new(stage_id.to_owned())?;
    ledger.append_event(ControlEvent::step(
        run_id,
        step_id,
        occurred_at_ms,
        ControlEventKind::CostObserved { observation },
    ))
}

/// Records a workflow stage gate result into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id or stage id is blank, or when
/// the ledger rejects the gate event append.
pub fn record_workflow_stage_gate_result(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    result: GateResult,
) -> ControlResult<ControlEventRecord> {
    let run_id = RunId::new(workflow_id.to_owned())?;
    let step_id = StepId::new(stage_id.to_owned())?;
    ledger.append_event(ControlEvent::step(
        run_id,
        step_id,
        occurred_at_ms,
        ControlEventKind::GateEvaluated { result },
    ))
}
