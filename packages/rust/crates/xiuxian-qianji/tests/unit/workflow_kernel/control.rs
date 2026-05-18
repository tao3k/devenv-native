use crate::workflow_kernel::{
    WorkflowControlEvidenceRequirements, WorkflowControlRecorder, WorkflowControlRecordingPolicy,
    WorkflowStageDecisionRecord, WorkflowStageFacts, WorkflowStageRecoveryDecisionRecord,
    WorkflowStageStatus, WorkflowStageTrace, WorkflowTrace, record_workflow_run_cost_observation,
    record_workflow_run_recovery_attempt, record_workflow_stage_cost_observation,
    record_workflow_stage_decision, record_workflow_stage_evidence,
    record_workflow_stage_gate_result, record_workflow_stage_recovery_attempt,
    record_workflow_stage_recovery_decision, record_workflow_trace_to_control_ledger,
    record_workflow_trace_to_control_ledger_with_required_evidence,
    workflow_trace_to_control_event_records,
    workflow_trace_to_control_event_records_with_required_evidence,
    workflow_trace_to_control_events,
};
use xiuxian_qianji_control::{
    ControlError, ControlEventKind, ControlLedger, CostObservation, EvidenceId, EvidenceRef,
    GateName, GateResult, InMemoryControlLedger, RecoveryAttempt, RecoveryPolicy, RunId, RunStatus,
    StepId, StepStatus,
};

#[test]
fn workflow_trace_maps_successful_stages_to_replayable_control_view() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.success".to_owned(),
        stages: vec![
            stage_trace(
                "load",
                WorkflowStageStatus::Succeeded,
                1_000,
                2_000_000,
                None,
            ),
            stage_trace(
                "render",
                WorkflowStageStatus::Succeeded,
                1_005,
                3_000_000,
                None,
            ),
        ],
    };

    let events = workflow_trace_to_control_events(&trace)?;
    assert_eq!(events.len(), 12);
    assert!(matches!(
        events[0].kind,
        ControlEventKind::RunCreated { .. }
    ));
    assert!(matches!(
        events[3].kind,
        ControlEventKind::StepCreated { .. }
    ));
    assert!(matches!(
        events[5].kind,
        ControlEventKind::ToolCallRecorded { .. }
    ));

    let view =
        xiuxian_qianji_control::replay_run_view(workflow_trace_to_control_event_records(&trace)?)?;
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(view.intent.as_deref(), Some("workflow:workflow.success"));
    assert_eq!(view.steps.len(), 2);
    assert!(
        view.steps
            .values()
            .all(|step| step.status == StepStatus::Succeeded)
    );
    assert_eq!(view.updated_at_ms, 1_008);
    Ok(())
}

#[test]
fn workflow_trace_required_evidence_projection_is_opt_in() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.required_evidence".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            1_000,
            1_000_000,
            None,
        )],
    };

    let default_view =
        xiuxian_qianji_control::replay_run_view(workflow_trace_to_control_event_records(&trace)?)?;
    let Some(default_step) = default_view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };
    assert!(default_step.required_evidence.is_empty());

    let requirements = WorkflowControlEvidenceRequirements::new()
        .require_stage_evidence("validate", ["validation_path", "authority"])?;
    let required_view = xiuxian_qianji_control::replay_run_view(
        workflow_trace_to_control_event_records_with_required_evidence(&trace, &requirements)?,
    )?;
    let Some(required_step) = required_view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(
        required_step.required_evidence,
        vec!["validation_path".to_owned(), "authority".to_owned()]
    );
    Ok(())
}

#[test]
fn workflow_trace_required_evidence_records_to_injected_ledger() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.required_evidence_ledger".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            1_100,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();
    let requirements = WorkflowControlEvidenceRequirements::new()
        .require_stage_evidence("validate", ["validation_path"])?;

    let records = record_workflow_trace_to_control_ledger_with_required_evidence(
        &ledger,
        &trace,
        &requirements,
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.required_evidence_ledger")?)?;
    let Some(step) = view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(records.len(), 8);
    assert_eq!(step.required_evidence, vec!["validation_path".to_owned()]);
    Ok(())
}

#[test]
fn workflow_trace_required_evidence_rejects_invalid_requirements() -> Result<(), ControlError> {
    assert!(matches!(
        WorkflowControlEvidenceRequirements::new().require_stage_evidence(" ", ["validation_path"]),
        Err(ControlError::BlankId { field: "step_id" })
    ));
    assert!(matches!(
        WorkflowControlEvidenceRequirements::new().require_stage_evidence("validate", [" "]),
        Err(ControlError::BlankId {
            field: "required_evidence"
        })
    ));

    let trace = WorkflowTrace {
        workflow_id: "workflow.required_evidence_invalid".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            1_200,
            1_000_000,
            None,
        )],
    };
    let requirements = WorkflowControlEvidenceRequirements::new()
        .require_stage_evidence("missing", ["validation_path"])?;

    assert!(matches!(
        workflow_trace_to_control_event_records_with_required_evidence(&trace, &requirements),
        Err(ControlError::InvalidEventSequence { .. })
    ));
    Ok(())
}

#[test]
fn workflow_trace_maps_failed_stage_to_failed_control_view() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.failed".to_owned(),
        stages: vec![stage_trace(
            "parse",
            WorkflowStageStatus::Failed,
            2_000,
            4_000_000,
            Some("parser rejected input"),
        )],
    };

    let view =
        xiuxian_qianji_control::replay_run_view(workflow_trace_to_control_event_records(&trace)?)?;

    assert_eq!(view.status, RunStatus::Failed);
    assert_eq!(view.steps.len(), 1);
    let Some(step) = view.steps.values().next() else {
        panic!("expected one failed step");
    };
    assert_eq!(step.status, StepStatus::Failed);
    assert_eq!(step.last_error.as_deref(), Some("parser rejected input"));
    Ok(())
}

#[test]
fn workflow_trace_records_to_injected_control_ledger() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.ledger".to_owned(),
        stages: vec![
            stage_trace(
                "collect",
                WorkflowStageStatus::Succeeded,
                3_000,
                1_000_000,
                None,
            ),
            stage_trace(
                "audit",
                WorkflowStageStatus::Succeeded,
                3_010,
                2_000_000,
                None,
            ),
        ],
    };
    let ledger = InMemoryControlLedger::new();

    let records = record_workflow_trace_to_control_ledger(&ledger, &trace)?;
    let view = ledger.load_run_view(&RunId::new("workflow.ledger")?)?;

    assert_eq!(records.len(), 12);
    assert_eq!(
        records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>()
    );
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(view.steps.len(), 2);
    assert!(
        view.steps
            .values()
            .all(|step| step.status == StepStatus::Succeeded)
    );
    Ok(())
}

#[test]
fn workflow_stage_recovery_attempt_replays_to_recovering_step() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.recovery".to_owned(),
        stages: vec![stage_trace(
            "parse",
            WorkflowStageStatus::Failed,
            7_000,
            1_000_000,
            Some("parser rejected input"),
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let attempt = recovery_attempt(1);
    let record = record_workflow_stage_recovery_attempt(
        &ledger,
        "workflow.recovery",
        "parse",
        7_050,
        attempt.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.recovery")?)?;
    let Some(step) = view.steps.get(&StepId::new("parse")?) else {
        panic!("expected parse step");
    };

    assert_eq!(record.sequence, 9);
    assert_eq!(view.status, RunStatus::Failed);
    assert_eq!(step.status, StepStatus::Recovering);
    assert_eq!(step.last_error.as_deref(), Some("parser rejected input"));
    assert_eq!(step.recovery_attempts, vec![attempt]);
    Ok(())
}

#[test]
fn workflow_run_recovery_attempt_replays_to_recovering_run() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.run_recovery".to_owned(),
        stages: vec![stage_trace(
            "parse",
            WorkflowStageStatus::Failed,
            7_100,
            1_000_000,
            Some("parser rejected input"),
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let attempt = recovery_attempt(1);
    let record = record_workflow_run_recovery_attempt(
        &ledger,
        "workflow.run_recovery",
        7_150,
        attempt.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.run_recovery")?)?;
    let Some(step) = view.steps.get(&StepId::new("parse")?) else {
        panic!("expected parse step");
    };

    assert_eq!(record.sequence, 9);
    assert_eq!(view.status, RunStatus::Recovering);
    assert_eq!(step.status, StepStatus::Failed);
    assert_eq!(step.last_error.as_deref(), Some("parser rejected input"));
    assert!(step.recovery_attempts.is_empty());
    Ok(())
}

#[test]
fn workflow_stage_recovery_attempt_rejects_blank_ids() {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_recovery_attempt(&ledger, " ", "parse", 0, recovery_attempt(1)),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_recovery_attempt(
            &ledger,
            "workflow.recovery",
            " ",
            0,
            recovery_attempt(1),
        ),
        Err(ControlError::BlankId { field: "step_id" })
    ));
}

#[test]
fn workflow_run_recovery_attempt_rejects_blank_ids() {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_run_recovery_attempt(&ledger, " ", 0, recovery_attempt(1)),
        Err(ControlError::BlankId { field: "run_id" })
    ));
}

#[test]
fn workflow_stage_evidence_replays_to_step_without_status_change() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.evidence".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            7_200,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let evidence = evidence_ref("evidence-validation-path", Some("validation_path"))?;
    let record = record_workflow_stage_evidence(
        &ledger,
        "workflow.evidence",
        "validate",
        7_250,
        evidence.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.evidence")?)?;
    let Some(step) = view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(record.sequence, 9);
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(step.status, StepStatus::Succeeded);
    assert_eq!(step.evidence, vec![evidence]);
    assert_eq!(
        step.covered_required_evidence(),
        vec!["validation_path".to_owned()]
    );
    Ok(())
}

#[test]
fn workflow_stage_evidence_rejects_blank_ids() -> Result<(), ControlError> {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_evidence(
            &ledger,
            " ",
            "validate",
            0,
            evidence_ref("evidence-validation-path", Some("validation_path"))?,
        ),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_evidence(
            &ledger,
            "workflow.evidence",
            " ",
            0,
            evidence_ref("evidence-validation-path", Some("validation_path"))?,
        ),
        Err(ControlError::BlankId { field: "step_id" })
    ));
    Ok(())
}

#[test]
fn workflow_cost_observations_replay_to_run_and_step_totals() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.cost".to_owned(),
        stages: vec![stage_trace(
            "infer",
            WorkflowStageStatus::Succeeded,
            7_300,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let run_observation = cost_observation("planner", 120, 30, 200);
    let step_observation = cost_observation("llm", 900, 180, 1_700);
    let run_record = record_workflow_run_cost_observation(
        &ledger,
        "workflow.cost",
        7_350,
        run_observation.clone(),
    )?;
    let step_record = record_workflow_stage_cost_observation(
        &ledger,
        "workflow.cost",
        "infer",
        7_360,
        step_observation.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.cost")?)?;
    let Some(step) = view.steps.get(&StepId::new("infer")?) else {
        panic!("expected infer step");
    };

    assert_eq!(run_record.sequence, 9);
    assert_eq!(step_record.sequence, 10);
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(step.status, StepStatus::Succeeded);
    assert_eq!(view.cost_observations, vec![run_observation]);
    assert_eq!(step.cost_observations, vec![step_observation]);
    assert_eq!(step.total_cost_usd_micros(), 1_700);
    assert_eq!(view.total_cost_usd_micros(), 1_900);
    Ok(())
}

#[test]
fn workflow_cost_observations_reject_blank_ids() {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_run_cost_observation(&ledger, " ", 0, cost_observation("planner", 1, 1, 1)),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_cost_observation(
            &ledger,
            " ",
            "infer",
            0,
            cost_observation("llm", 1, 1, 1),
        ),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_cost_observation(
            &ledger,
            "workflow.cost",
            " ",
            0,
            cost_observation("llm", 1, 1, 1),
        ),
        Err(ControlError::BlankId { field: "step_id" })
    ));
}

#[test]
fn workflow_stage_gate_result_replays_to_step_without_status_change() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.gate".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            7_400,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let result = gate_result("required-evidence", true)?;
    let record = record_workflow_stage_gate_result(
        &ledger,
        "workflow.gate",
        "validate",
        7_450,
        result.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.gate")?)?;
    let Some(step) = view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(record.sequence, 9);
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(step.status, StepStatus::Succeeded);
    assert_eq!(step.gate_results, vec![result]);
    Ok(())
}

#[test]
fn workflow_stage_gate_result_rejects_blank_ids() -> Result<(), ControlError> {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_gate_result(
            &ledger,
            " ",
            "validate",
            0,
            gate_result("required-evidence", false)?,
        ),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_gate_result(
            &ledger,
            "workflow.gate",
            " ",
            0,
            gate_result("required-evidence", false)?,
        ),
        Err(ControlError::BlankId { field: "step_id" })
    ));
    Ok(())
}

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

#[test]
fn workflow_control_recorder_rejects_existing_run_by_default() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.duplicate".to_owned(),
        stages: vec![stage_trace(
            "collect",
            WorkflowStageStatus::Succeeded,
            4_000,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();
    let first = WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let second = WorkflowControlRecorder::new(&ledger).record_trace(&trace);

    assert_eq!(first.run_id.as_str(), "workflow.duplicate");
    assert_eq!(first.terminal_status, RunStatus::Completed);
    assert_eq!(first.appended_event_count, 8);
    assert_eq!(first.run_view.status, RunStatus::Completed);
    assert_eq!(first.run_view.steps.len(), 1);
    assert!(matches!(
        second,
        Err(ControlError::InvalidEventSequence { .. })
    ));
    assert_eq!(
        ledger
            .load_events(&RunId::new("workflow.duplicate")?)?
            .len(),
        8
    );
    Ok(())
}

#[test]
fn workflow_control_recorder_supports_explicit_append_only_mode() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.append_only".to_owned(),
        stages: vec![stage_trace(
            "collect",
            WorkflowStageStatus::Succeeded,
            5_000,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();
    let recorder = WorkflowControlRecorder::new(&ledger)
        .with_policy(WorkflowControlRecordingPolicy::AppendOnly);

    let first = recorder.record_trace(&trace)?;
    let second = recorder.record_trace(&trace)?;
    let records = ledger.load_events(&RunId::new("workflow.append_only")?)?;

    assert_eq!(first.appended_event_count, 8);
    assert_eq!(second.appended_event_count, 8);
    assert_eq!(second.run_view.status, RunStatus::Completed);
    assert_eq!(second.run_view.steps.len(), 1);
    assert_eq!(records.len(), 16);
    assert_eq!(
        records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        (1..=16).collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn workflow_control_recorder_can_reuse_existing_run_without_appending() -> Result<(), ControlError>
{
    let trace = WorkflowTrace {
        workflow_id: "workflow.reuse_existing".to_owned(),
        stages: vec![stage_trace(
            "collect",
            WorkflowStageStatus::Succeeded,
            6_000,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();

    let first = WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let second = WorkflowControlRecorder::new(&ledger)
        .with_policy(WorkflowControlRecordingPolicy::ReuseExistingRun)
        .record_trace(&trace)?;
    let records = ledger.load_events(&RunId::new("workflow.reuse_existing")?)?;

    assert_eq!(first.appended_event_count, 8);
    assert_eq!(second.appended_event_count, 0);
    assert!(second.records.is_empty());
    assert_eq!(second.run_view.status, RunStatus::Completed);
    assert_eq!(second.run_view.steps.len(), 1);
    assert_eq!(records.len(), 8);
    assert_eq!(
        records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        (1..=8).collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn workflow_trace_rejects_blank_control_ids() {
    let trace = WorkflowTrace {
        workflow_id: " ".to_owned(),
        stages: Vec::new(),
    };

    assert!(matches!(
        workflow_trace_to_control_events(&trace),
        Err(ControlError::BlankId { field: "run_id" })
    ));
}

fn recovery_attempt(attempt: u32) -> RecoveryAttempt {
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

fn evidence_ref(
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

fn cost_observation(
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

fn gate_result(gate_name: &str, passed: bool) -> Result<GateResult, ControlError> {
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

fn decision_record() -> Result<WorkflowStageDecisionRecord, ControlError> {
    Ok(WorkflowStageDecisionRecord {
        evidence: vec![evidence_ref(
            "evidence-validation-path",
            Some("validation_path"),
        )?],
        gate_results: vec![gate_result("required-evidence", true)?],
        cost_observations: vec![cost_observation("llm", 100, 20, 250)],
    })
}

fn recovery_decision_record(
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

fn stage_trace(
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
