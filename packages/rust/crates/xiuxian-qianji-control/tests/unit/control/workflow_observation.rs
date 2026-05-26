use std::error::Error;

use serde_json::Value;
use xiuxian_qianji_control::{
    ControlError, ControlEventKind, ControlLedger, CostObservation, EvidenceId, EvidenceRef,
    GateName, GateResult, InMemoryControlLedger, RecoveryAttempt, RecoveryPolicy, RunId, StepId,
    WorkflowControlEvidenceRequirements, WorkflowStageDecisionRecord,
    WorkflowStageDecisionRecordingRequest, WorkflowStageEvidenceRecordingRequest,
    WorkflowStageRecoveryDecisionRecord, WorkflowStageRecoveryDecisionRecordingRequest,
    record_workflow_stage_decision, record_workflow_stage_evidence,
    record_workflow_stage_recovery_decision,
};

#[test]
fn workflow_control_evidence_requirements_normalize_step_keys() -> Result<(), Box<dyn Error>> {
    let step_id = StepId::new("validate")?;
    let requirements = WorkflowControlEvidenceRequirements::new()
        .require_stage_evidence("validate", [" authority ", "validation_path", "authority"])?;

    assert_eq!(
        requirements.required_evidence_for_step(&step_id),
        vec!["authority", "validation_path"]
    );
    assert_eq!(
        requirements
            .step_ids()
            .map(StepId::as_str)
            .collect::<Vec<_>>(),
        vec!["validate"]
    );
    Ok(())
}

#[test]
fn workflow_control_evidence_requirements_reject_blank_keys() {
    let Err(error) =
        WorkflowControlEvidenceRequirements::new().require_stage_evidence("validate", [" "])
    else {
        panic!("blank evidence key should fail");
    };

    assert!(matches!(
        error,
        ControlError::BlankId {
            field: "required_evidence"
        }
    ));
}

#[test]
fn record_workflow_stage_evidence_appends_control_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("workflow.observation")?;
    let step_id = StepId::new("validate")?;

    record_workflow_stage_evidence(WorkflowStageEvidenceRecordingRequest {
        ledger: &ledger,
        run_id: run_id.clone(),
        step_id: step_id.clone(),
        occurred_at_ms: 42,
        evidence: evidence_ref("validation")?,
    })?;

    let records = ledger.load_events(&run_id)?;
    assert_eq!(records.len(), 1);
    assert!(matches!(
        records[0].event.kind,
        ControlEventKind::EvidenceAttached { .. }
    ));
    assert_eq!(records[0].event.step_id.as_ref(), Some(&step_id));
    Ok(())
}

#[test]
fn record_workflow_stage_decision_appends_deterministic_fact_order() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("workflow.decision")?;
    let step_id = StepId::new("validate")?;

    let outcome = record_workflow_stage_decision(WorkflowStageDecisionRecordingRequest {
        ledger: &ledger,
        run_id: run_id.clone(),
        step_id,
        occurred_at_ms: 77,
        decision: decision_record(true)?,
    })?;

    assert_eq!(outcome.appended_event_count, 3);
    let event_names = outcome
        .records
        .iter()
        .map(|record| match &record.event.kind {
            ControlEventKind::EvidenceAttached { .. } => "evidence",
            ControlEventKind::GateEvaluated { .. } => "gate",
            ControlEventKind::CostObserved { .. } => "cost",
            other => panic!("unexpected event kind: {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(event_names, vec!["evidence", "gate", "cost"]);
    assert_eq!(outcome.run_id, run_id);
    Ok(())
}

#[test]
fn record_workflow_stage_recovery_decision_requires_failed_gate() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let Err(error) =
        record_workflow_stage_recovery_decision(WorkflowStageRecoveryDecisionRecordingRequest {
            ledger: &ledger,
            run_id: RunId::new("workflow.recovery")?,
            step_id: StepId::new("validate")?,
            occurred_at_ms: 77,
            recovery: WorkflowStageRecoveryDecisionRecord {
                decision: decision_record(true)?,
                recovery_attempt: recovery_attempt(),
            },
        })
    else {
        return Err("passing gate should not admit recovery".into());
    };

    assert!(error.to_string().contains("requires a failed gate result"));
    Ok(())
}

#[test]
fn record_workflow_stage_recovery_decision_appends_recovery_after_facts()
-> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let outcome =
        record_workflow_stage_recovery_decision(WorkflowStageRecoveryDecisionRecordingRequest {
            ledger: &ledger,
            run_id: RunId::new("workflow.recovery.failed")?,
            step_id: StepId::new("validate")?,
            occurred_at_ms: 77,
            recovery: WorkflowStageRecoveryDecisionRecord {
                decision: decision_record(false)?,
                recovery_attempt: recovery_attempt(),
            },
        })?;

    let event_names = outcome
        .records
        .iter()
        .map(|record| match &record.event.kind {
            ControlEventKind::EvidenceAttached { .. } => "evidence",
            ControlEventKind::GateEvaluated { .. } => "gate",
            ControlEventKind::CostObserved { .. } => "cost",
            ControlEventKind::RecoveryStarted { .. } => "recovery",
            other => panic!("unexpected event kind: {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(event_names, vec!["evidence", "gate", "cost", "recovery"]);
    Ok(())
}

fn decision_record(gate_passed: bool) -> Result<WorkflowStageDecisionRecord, Box<dyn Error>> {
    Ok(WorkflowStageDecisionRecord {
        evidence: vec![evidence_ref("authority")?],
        gate_results: vec![gate_result(gate_passed)?],
        cost_observations: vec![CostObservation {
            provider: "local".to_owned(),
            prompt_tokens: 3,
            completion_tokens: 5,
            ..CostObservation::default()
        }],
    })
}

fn evidence_ref(suffix: &str) -> Result<EvidenceRef, Box<dyn Error>> {
    Ok(EvidenceRef {
        evidence_id: EvidenceId::new(format!("evidence.{suffix}"))?,
        requirement_key: Some(suffix.to_owned()),
        source: "unit-test".to_owned(),
        uri: None,
        summary: Some(format!("{suffix} evidence")),
        metadata: Value::Null,
    })
}

fn gate_result(passed: bool) -> Result<GateResult, Box<dyn Error>> {
    Ok(GateResult {
        gate_name: GateName::new("required-evidence")?,
        passed,
        required_evidence_covered: passed,
        selected_required_evidence: if passed {
            vec!["authority".to_owned()]
        } else {
            Vec::new()
        },
        missing_required_evidence: if passed {
            Vec::new()
        } else {
            vec!["authority".to_owned()]
        },
        reasons: Vec::new(),
        metadata: Value::Null,
    })
}

fn recovery_attempt() -> RecoveryAttempt {
    RecoveryAttempt {
        attempt: 1,
        reason: "missing authority evidence".to_owned(),
        policy: RecoveryPolicy {
            max_attempts: 2,
            backoff_ms: 10,
            require_human_approval: false,
        },
    }
}
