//! Workflow-neutral helpers for recording step observations into a control ledger.

use std::collections::BTreeMap;

use crate::{
    ControlError, ControlEventRecord, ControlLedger, ControlResult, CostObservation,
    CostObservationJournalRecord, EvidenceRef, GateResult, RecoveryAttempt, RecoveryItemScope,
    RecoveryStartedJournalRecord, RunId, StepEvidenceJournalRecord, StepGateResultJournalRecord,
    StepId, record_cost_observation, record_recovery_started, record_step_evidence,
    record_step_gate_result,
};

/// Workflow-side required-evidence declarations for control projection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowControlEvidenceRequirements {
    required_by_step: BTreeMap<StepId, Vec<String>>,
}

impl WorkflowControlEvidenceRequirements {
    /// Creates an empty requirements set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            required_by_step: BTreeMap::new(),
        }
    }

    /// Returns true when no step requirements are declared.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.required_by_step.is_empty()
    }

    /// Adds required-evidence keys for one workflow stage.
    ///
    /// # Errors
    ///
    /// Returns a control error when the step id or any required-evidence key
    /// is blank.
    pub fn require_stage_evidence<I, S>(
        mut self,
        stage_id: impl Into<String>,
        required_evidence: I,
    ) -> ControlResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.insert_stage_evidence(stage_id, required_evidence)?;
        Ok(self)
    }

    /// Inserts or replaces required-evidence keys for one workflow stage.
    ///
    /// # Errors
    ///
    /// Returns a control error when the step id or any required-evidence key
    /// is blank.
    pub fn insert_stage_evidence<I, S>(
        &mut self,
        stage_id: impl Into<String>,
        required_evidence: I,
    ) -> ControlResult<()>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let step_id = StepId::new(stage_id)?;
        let required_evidence = normalize_required_evidence(required_evidence)?;
        self.required_by_step.insert(step_id, required_evidence);
        Ok(())
    }

    /// Returns required-evidence keys for one control step.
    #[must_use]
    pub fn required_evidence_for_step(&self, step_id: &StepId) -> Vec<String> {
        self.required_by_step
            .get(step_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Iterates over step ids with declared requirements.
    pub fn step_ids(&self) -> impl Iterator<Item = &StepId> {
        self.required_by_step.keys()
    }
}

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
    record_recovery_started(
        ledger,
        RecoveryStartedJournalRecord::new(run_id, RecoveryItemScope::Run, attempt, occurred_at_ms),
    )
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
    record_cost_observation(
        ledger,
        CostObservationJournalRecord::run(run_id, observation, occurred_at_ms),
    )
}

/// Records a workflow stage recovery attempt into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id or step id is blank, or when
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
    record_recovery_started(
        ledger,
        RecoveryStartedJournalRecord::new(
            run_id,
            RecoveryItemScope::Step { step_id },
            attempt,
            occurred_at_ms,
        ),
    )
}

/// Records workflow stage evidence into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id or step id is blank, or when
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
    record_step_evidence(
        ledger,
        StepEvidenceJournalRecord::new(run_id, step_id, evidence, occurred_at_ms),
    )
}

/// Records a workflow stage cost observation into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id or step id is blank, or when
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
    record_cost_observation(
        ledger,
        CostObservationJournalRecord::step(run_id, step_id, observation, occurred_at_ms),
    )
}

/// Records a workflow stage gate result into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when the workflow id or step id is blank, or when
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
    record_step_gate_result(
        ledger,
        StepGateResultJournalRecord::new(run_id, step_id, result, occurred_at_ms),
    )
}

fn normalize_required_evidence<I, S>(required_evidence: I) -> ControlResult<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    required_evidence
        .into_iter()
        .try_fold(Vec::new(), |normalized, key| {
            Ok(append_unique_required_evidence_key(
                normalized,
                normalize_required_evidence_key(key)?,
            ))
        })
}

fn normalize_required_evidence_key<S>(key: S) -> ControlResult<String>
where
    S: Into<String>,
{
    let key = key.into().trim().to_owned();
    if key.is_empty() {
        return Err(ControlError::BlankId {
            field: "required_evidence",
        });
    }
    Ok(key)
}

fn append_unique_required_evidence_key(mut normalized: Vec<String>, key: String) -> Vec<String> {
    if !normalized.iter().any(|existing| existing == &key) {
        normalized.push(key);
    }
    normalized
}
