use crate::workflow_kernel::{
    WorkflowRunCostObservationRecordingRequest, WorkflowRunRecoveryAttemptRecordingRequest,
    WorkflowStageCostObservationRecordingRequest, WorkflowStageDecisionRecord,
    WorkflowStageDecisionRecordingOutcome, WorkflowStageDecisionRecordingRequest,
    WorkflowStageEvidenceRecordingRequest, WorkflowStageFacts,
    WorkflowStageGateResultRecordingRequest, WorkflowStageRecoveryAttemptRecordingRequest,
    WorkflowStageRecoveryDecisionRecord, WorkflowStageRecoveryDecisionRecordingRequest,
    WorkflowStageStatus, WorkflowStageTrace,
    record_workflow_run_cost_observation as record_run_cost_observation,
    record_workflow_run_recovery_attempt as record_run_recovery_attempt,
    record_workflow_stage_cost_observation as record_stage_cost_observation,
    record_workflow_stage_decision as record_stage_decision,
    record_workflow_stage_evidence as record_stage_evidence,
    record_workflow_stage_gate_result as record_stage_gate_result,
    record_workflow_stage_recovery_attempt as record_stage_recovery_attempt,
    record_workflow_stage_recovery_decision as record_stage_recovery_decision,
};
use xiuxian_qianji_control::{
    ControlError, ControlEventRecord, ControlLedger, ControlResult, CostObservation, EvidenceId,
    EvidenceRef, GateName, GateResult, RecoveryAttempt, RecoveryPolicy, RunId, StepId,
};

pub(super) fn record_workflow_run_recovery_attempt(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    occurred_at_ms: u64,
    attempt: RecoveryAttempt,
) -> ControlResult<ControlEventRecord> {
    record_run_recovery_attempt(WorkflowRunRecoveryAttemptRecordingRequest {
        ledger,
        run_id: RunId::new(workflow_id.to_owned())?,
        occurred_at_ms,
        attempt,
    })
}

pub(super) fn record_workflow_run_cost_observation(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    occurred_at_ms: u64,
    observation: CostObservation,
) -> ControlResult<ControlEventRecord> {
    record_run_cost_observation(WorkflowRunCostObservationRecordingRequest {
        ledger,
        run_id: RunId::new(workflow_id.to_owned())?,
        occurred_at_ms,
        observation,
    })
}

pub(super) fn record_workflow_stage_recovery_attempt(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    attempt: RecoveryAttempt,
) -> ControlResult<ControlEventRecord> {
    record_stage_recovery_attempt(WorkflowStageRecoveryAttemptRecordingRequest {
        ledger,
        run_id: RunId::new(workflow_id.to_owned())?,
        step_id: StepId::new(stage_id.to_owned())?,
        occurred_at_ms,
        attempt,
    })
}

pub(super) fn record_workflow_stage_evidence(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    evidence: EvidenceRef,
) -> ControlResult<ControlEventRecord> {
    record_stage_evidence(WorkflowStageEvidenceRecordingRequest {
        ledger,
        run_id: RunId::new(workflow_id.to_owned())?,
        step_id: StepId::new(stage_id.to_owned())?,
        occurred_at_ms,
        evidence,
    })
}

pub(super) fn record_workflow_stage_cost_observation(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    observation: CostObservation,
) -> ControlResult<ControlEventRecord> {
    record_stage_cost_observation(WorkflowStageCostObservationRecordingRequest {
        ledger,
        run_id: RunId::new(workflow_id.to_owned())?,
        step_id: StepId::new(stage_id.to_owned())?,
        occurred_at_ms,
        observation,
    })
}

pub(super) fn record_workflow_stage_gate_result(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    result: GateResult,
) -> ControlResult<ControlEventRecord> {
    record_stage_gate_result(WorkflowStageGateResultRecordingRequest {
        ledger,
        run_id: RunId::new(workflow_id.to_owned())?,
        step_id: StepId::new(stage_id.to_owned())?,
        occurred_at_ms,
        result,
    })
}

pub(super) fn record_workflow_stage_decision(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    decision: WorkflowStageDecisionRecord,
) -> ControlResult<WorkflowStageDecisionRecordingOutcome> {
    record_stage_decision(WorkflowStageDecisionRecordingRequest {
        ledger,
        run_id: RunId::new(workflow_id.to_owned())?,
        step_id: StepId::new(stage_id.to_owned())?,
        occurred_at_ms,
        decision,
    })
}

pub(super) fn record_workflow_stage_recovery_decision(
    ledger: &dyn ControlLedger,
    workflow_id: &str,
    stage_id: &str,
    occurred_at_ms: u64,
    recovery: WorkflowStageRecoveryDecisionRecord,
) -> ControlResult<WorkflowStageDecisionRecordingOutcome> {
    record_stage_recovery_decision(WorkflowStageRecoveryDecisionRecordingRequest {
        ledger,
        run_id: RunId::new(workflow_id.to_owned())?,
        step_id: StepId::new(stage_id.to_owned())?,
        occurred_at_ms,
        recovery,
    })
}

pub(super) fn recovery_attempt(attempt: u32) -> RecoveryAttempt {
    RecoveryAttempt {
        attempt,
        reason: "retry failed parser stage".to_owned(),
        policy: RecoveryPolicy {
            max_attempts: 3,
            backoff_ms: 250,
            require_human_approval: false,
        },
    }
}

pub(super) fn evidence_ref(
    evidence_id: &str,
    requirement_key: Option<&str>,
) -> Result<EvidenceRef, ControlError> {
    Ok(EvidenceRef {
        evidence_id: EvidenceId::new(evidence_id)?,
        requirement_key: requirement_key.map(str::to_owned),
        source: "workflow-kernel-test".to_owned(),
        uri: Some("artifact://validation-path".to_owned()),
        summary: Some("Validation path was checked".to_owned()),
        metadata: serde_json::json!({
            "source": "workflow_kernel_control_test",
        }),
    })
}

pub(super) fn cost_observation(
    provider: &str,
    prompt_tokens: u64,
    completion_tokens: u64,
    cost_usd_micros: u64,
) -> CostObservation {
    CostObservation {
        provider: provider.to_owned(),
        model: Some("unit-model".to_owned()),
        prompt_tokens,
        completion_tokens,
        total_tokens: None,
        cost_usd_micros,
        latency_ms: Some(25),
    }
}

pub(super) fn gate_result(gate_name: &str, passed: bool) -> Result<GateResult, ControlError> {
    Ok(GateResult {
        gate_name: GateName::new(gate_name)?,
        passed,
        required_evidence_covered: passed,
        selected_required_evidence: if passed {
            vec!["validation_path".to_owned()]
        } else {
            Vec::new()
        },
        missing_required_evidence: if passed {
            Vec::new()
        } else {
            vec!["validation_path".to_owned()]
        },
        reasons: if passed {
            Vec::new()
        } else {
            vec!["missing required evidence: validation_path".to_owned()]
        },
        metadata: serde_json::json!({
            "source": "workflow_kernel_control_test",
        }),
    })
}

pub(super) fn decision_record() -> Result<WorkflowStageDecisionRecord, ControlError> {
    Ok(WorkflowStageDecisionRecord {
        evidence: vec![evidence_ref(
            "evidence-validation-path",
            Some("validation_path"),
        )?],
        gate_results: vec![gate_result("required-evidence", true)?],
        cost_observations: vec![cost_observation("llm", 100, 20, 250)],
    })
}

pub(super) fn recovery_decision_record(
    gate_passed: bool,
) -> Result<WorkflowStageRecoveryDecisionRecord, ControlError> {
    Ok(WorkflowStageRecoveryDecisionRecord {
        decision: WorkflowStageDecisionRecord {
            evidence: vec![evidence_ref(
                "evidence-validation-path",
                Some("validation_path"),
            )?],
            gate_results: vec![gate_result("required-evidence", gate_passed)?],
            cost_observations: vec![cost_observation("llm", 100, 20, 250)],
        },
        recovery_attempt: recovery_attempt(1),
    })
}

pub(super) fn stage_trace(
    stage_id: &str,
    status: WorkflowStageStatus,
    started_unix_ms: u64,
    duration_nanos: u64,
    error: Option<&str>,
) -> WorkflowStageTrace {
    WorkflowStageTrace {
        stage_id: stage_id.to_owned(),
        status,
        started_unix_ms,
        duration_nanos,
        input: WorkflowStageFacts::typed("input").with_item_count(1),
        output: WorkflowStageFacts::typed("output").with_item_count(1),
        error: error.map(str::to_owned),
        checkpoints: Vec::new(),
    }
}
