use super::support::{
    cost_observation, decision_record, evidence_ref, gate_result, record_workflow_stage_decision,
    record_workflow_stage_recovery_decision, recovery_attempt, recovery_decision_record,
    stage_trace,
};
use crate::workflow_kernel::{
    WorkflowControlRecorder, WorkflowStageDecisionRecord, WorkflowStageRecoveryDecisionRecord,
    WorkflowStageStatus, WorkflowTrace,
};
use xiuxian_qianji_control::{
    ControlError, ControlEventKind, InMemoryControlLedger, RunStatus, StepId, StepStatus,
};

#[test]
fn workflow_stage_decision_records_facts_in_stable_order() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.decision".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            7_500,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let evidence = evidence_ref("evidence-validation-path", Some("validation_path"))?;
    let gate_result = gate_result("required-evidence", true)?;
    let cost = cost_observation("llm", 100, 20, 250);
    let outcome = record_workflow_stage_decision(
        &ledger,
        "workflow.decision",
        "validate",
        7_550,
        WorkflowStageDecisionRecord {
            evidence: vec![evidence.clone()],
            gate_results: vec![gate_result.clone()],
            cost_observations: vec![cost.clone()],
        },
    )?;
    let Some(step) = outcome.run_view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(outcome.run_id.as_str(), "workflow.decision");
    assert_eq!(outcome.step_id.as_str(), "validate");
    assert_eq!(outcome.appended_event_count, 3);
    assert_eq!(
        outcome
            .records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        vec![9, 10, 11]
    );
    assert!(matches!(
        outcome.records[0].event.kind,
        ControlEventKind::EvidenceAttached { .. }
    ));
    assert!(matches!(
        outcome.records[1].event.kind,
        ControlEventKind::GateEvaluated { .. }
    ));
    assert!(matches!(
        outcome.records[2].event.kind,
        ControlEventKind::CostObserved { .. }
    ));
    assert_eq!(outcome.run_view.status, RunStatus::Completed);
    assert_eq!(step.status, StepStatus::Succeeded);
    assert_eq!(step.evidence, vec![evidence]);
    assert_eq!(step.gate_results, vec![gate_result]);
    assert_eq!(step.cost_observations, vec![cost]);
    assert_eq!(outcome.run_view.total_cost_usd_micros(), 250);
    Ok(())
}

#[test]
fn workflow_stage_decision_rejects_blank_ids() -> Result<(), ControlError> {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_decision(&ledger, " ", "validate", 0, decision_record()?,),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_decision(&ledger, "workflow.decision", " ", 0, decision_record()?,),
        Err(ControlError::BlankId { field: "step_id" })
    ));
    Ok(())
}

#[test]
fn workflow_stage_decision_rejects_empty_record() {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_decision(
            &ledger,
            "workflow.decision",
            "validate",
            0,
            WorkflowStageDecisionRecord::default(),
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));
}

#[test]
fn workflow_stage_recovery_decision_records_failed_gate_then_recovery() -> Result<(), ControlError>
{
    let trace = WorkflowTrace {
        workflow_id: "workflow.recovery_decision".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Failed,
            7_600,
            1_000_000,
            Some("required evidence missing"),
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let evidence = evidence_ref("evidence-validation-path", Some("validation_path"))?;
    let gate_result = gate_result("required-evidence", false)?;
    let cost = cost_observation("llm", 100, 20, 250);
    let attempt = recovery_attempt(1);
    let outcome = record_workflow_stage_recovery_decision(
        &ledger,
        "workflow.recovery_decision",
        "validate",
        7_650,
        WorkflowStageRecoveryDecisionRecord {
            decision: WorkflowStageDecisionRecord {
                evidence: vec![evidence.clone()],
                gate_results: vec![gate_result.clone()],
                cost_observations: vec![cost.clone()],
            },
            recovery_attempt: attempt.clone(),
        },
    )?;
    let Some(step) = outcome.run_view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(outcome.appended_event_count, 4);
    assert_eq!(
        outcome
            .records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        vec![9, 10, 11, 12]
    );
    assert!(matches!(
        outcome.records[0].event.kind,
        ControlEventKind::EvidenceAttached { .. }
    ));
    assert!(matches!(
        outcome.records[1].event.kind,
        ControlEventKind::GateEvaluated { .. }
    ));
    assert!(matches!(
        outcome.records[2].event.kind,
        ControlEventKind::CostObserved { .. }
    ));
    assert!(matches!(
        outcome.records[3].event.kind,
        ControlEventKind::RecoveryStarted { .. }
    ));
    assert_eq!(outcome.run_view.status, RunStatus::Failed);
    assert_eq!(step.status, StepStatus::Recovering);
    assert_eq!(
        step.last_error.as_deref(),
        Some("required evidence missing")
    );
    assert_eq!(step.evidence, vec![evidence]);
    assert_eq!(step.gate_results, vec![gate_result]);
    assert_eq!(step.cost_observations, vec![cost]);
    assert_eq!(step.recovery_attempts, vec![attempt]);
    Ok(())
}

#[test]
fn workflow_stage_recovery_decision_rejects_blank_ids() -> Result<(), ControlError> {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_recovery_decision(
            &ledger,
            " ",
            "validate",
            0,
            recovery_decision_record(false)?,
        ),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_recovery_decision(
            &ledger,
            "workflow.recovery_decision",
            " ",
            0,
            recovery_decision_record(false)?,
        ),
        Err(ControlError::BlankId { field: "step_id" })
    ));
    Ok(())
}

#[test]
fn workflow_stage_recovery_decision_rejects_successful_gate() -> Result<(), ControlError> {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_recovery_decision(
            &ledger,
            "workflow.recovery_decision",
            "validate",
            0,
            recovery_decision_record(true)?,
        ),
        Err(ControlError::InvalidEventSequence { .. })
    ));
    Ok(())
}
