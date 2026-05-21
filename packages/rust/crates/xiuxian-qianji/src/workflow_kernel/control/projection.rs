//! Projects targeted workflow control observations into the Qianji ledger.

use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    CostObservation, EvidenceRef, GateResult, RecoveryAttempt, RunId, StepId,
};

/// Request for recording a run-level recovery attempt.
pub struct WorkflowRunRecoveryAttemptRecordingRequest<'ledger> {
    /// Ledger that owns the appended control event.
    pub ledger: &'ledger dyn ControlLedger,
    /// Control-plane run id for the workflow.
    pub run_id: RunId,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Recovery attempt to append.
    pub attempt: RecoveryAttempt,
}

/// Request for recording a run-level cost observation.
pub struct WorkflowRunCostObservationRecordingRequest<'ledger> {
    /// Ledger that owns the appended control event.
    pub ledger: &'ledger dyn ControlLedger,
    /// Control-plane run id for the workflow.
    pub run_id: RunId,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Cost observation to append.
    pub observation: CostObservation,
}

/// Request for recording a stage-level recovery attempt.
pub struct WorkflowStageRecoveryAttemptRecordingRequest<'ledger> {
    /// Ledger that owns the appended control event.
    pub ledger: &'ledger dyn ControlLedger,
    /// Control-plane run id for the workflow.
    pub run_id: RunId,
    /// Control-plane step id for the workflow stage.
    pub step_id: StepId,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Recovery attempt to append.
    pub attempt: RecoveryAttempt,
}

/// Request for recording stage-level evidence.
pub struct WorkflowStageEvidenceRecordingRequest<'ledger> {
    /// Ledger that owns the appended control event.
    pub ledger: &'ledger dyn ControlLedger,
    /// Control-plane run id for the workflow.
    pub run_id: RunId,
    /// Control-plane step id for the workflow stage.
    pub step_id: StepId,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Evidence reference to append.
    pub evidence: EvidenceRef,
}

/// Request for recording a stage-level cost observation.
pub struct WorkflowStageCostObservationRecordingRequest<'ledger> {
    /// Ledger that owns the appended control event.
    pub ledger: &'ledger dyn ControlLedger,
    /// Control-plane run id for the workflow.
    pub run_id: RunId,
    /// Control-plane step id for the workflow stage.
    pub step_id: StepId,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Cost observation to append.
    pub observation: CostObservation,
}

/// Request for recording a stage-level gate result.
pub struct WorkflowStageGateResultRecordingRequest<'ledger> {
    /// Ledger that owns the appended control event.
    pub ledger: &'ledger dyn ControlLedger,
    /// Control-plane run id for the workflow.
    pub run_id: RunId,
    /// Control-plane step id for the workflow stage.
    pub step_id: StepId,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Gate result to append.
    pub result: GateResult,
}

/// Records a workflow run recovery attempt into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id is blank, or when the ledger
/// rejects the recovery event append.
pub fn record_workflow_run_recovery_attempt(
    request: WorkflowRunRecoveryAttemptRecordingRequest<'_>,
) -> ControlResult<ControlEventRecord> {
    let WorkflowRunRecoveryAttemptRecordingRequest {
        ledger,
        run_id,
        occurred_at_ms,
        attempt,
    } = request;
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
    request: WorkflowRunCostObservationRecordingRequest<'_>,
) -> ControlResult<ControlEventRecord> {
    let WorkflowRunCostObservationRecordingRequest {
        ledger,
        run_id,
        occurred_at_ms,
        observation,
    } = request;
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
    request: WorkflowStageRecoveryAttemptRecordingRequest<'_>,
) -> ControlResult<ControlEventRecord> {
    let WorkflowStageRecoveryAttemptRecordingRequest {
        ledger,
        run_id,
        step_id,
        occurred_at_ms,
        attempt,
    } = request;
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
    request: WorkflowStageEvidenceRecordingRequest<'_>,
) -> ControlResult<ControlEventRecord> {
    let WorkflowStageEvidenceRecordingRequest {
        ledger,
        run_id,
        step_id,
        occurred_at_ms,
        evidence,
    } = request;
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
    request: WorkflowStageCostObservationRecordingRequest<'_>,
) -> ControlResult<ControlEventRecord> {
    let WorkflowStageCostObservationRecordingRequest {
        ledger,
        run_id,
        step_id,
        occurred_at_ms,
        observation,
    } = request;
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
    request: WorkflowStageGateResultRecordingRequest<'_>,
) -> ControlResult<ControlEventRecord> {
    let WorkflowStageGateResultRecordingRequest {
        ledger,
        run_id,
        step_id,
        occurred_at_ms,
        result,
    } = request;
    ledger.append_event(ControlEvent::step(
        run_id,
        step_id,
        occurred_at_ms,
        ControlEventKind::GateEvaluated { result },
    ))
}
