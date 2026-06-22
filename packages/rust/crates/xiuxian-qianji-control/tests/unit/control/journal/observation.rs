use std::error::Error;

use serde_json::Value;
use xiuxian_qianji_control::{
    ControlEventKind, ControlLedger, CostObservation, CostObservationJournalRecord, EvidenceId,
    EvidenceRef, GateName, GateResult, InMemoryControlLedger, RunCreatedJournalRecord, RunId,
    StepCreatedJournalRecord, StepEvidenceJournalRecord, StepGateResultJournalRecord, StepId,
    record_cost_observation, record_run_created, record_step_created, record_step_evidence,
    record_step_gate_result,
};

#[test]
fn observation_journal_records_step_evidence() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("observation.evidence")?;
    let step_id = StepId::new("validate")?;

    seed_step(&ledger, &run_id, &step_id)?;
    let record = record_step_evidence(
        &ledger,
        StepEvidenceJournalRecord::new(
            run_id.clone(),
            step_id.clone(),
            evidence_ref("authority")?,
            12,
        ),
    )?;

    assert!(matches!(
        record.event.kind,
        ControlEventKind::EvidenceAttached { .. }
    ));
    let view = ledger.load_run_view(&run_id)?;
    let Some(step) = view.steps.get(&step_id) else {
        panic!("expected validate step");
    };
    assert_eq!(step.evidence.len(), 1);
    assert_eq!(
        step.evidence[0].requirement_key.as_deref(),
        Some("authority")
    );
    Ok(())
}

#[test]
fn observation_journal_records_step_gate_result() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("observation.gate")?;
    let step_id = StepId::new("validate")?;

    seed_step(&ledger, &run_id, &step_id)?;
    let record = record_step_gate_result(
        &ledger,
        StepGateResultJournalRecord::new(run_id.clone(), step_id.clone(), gate_result(false)?, 13),
    )?;

    assert!(matches!(
        record.event.kind,
        ControlEventKind::GateEvaluated { .. }
    ));
    let view = ledger.load_run_view(&run_id)?;
    let Some(step) = view.steps.get(&step_id) else {
        panic!("expected validate step");
    };
    assert_eq!(step.gate_results.len(), 1);
    assert_eq!(
        step.gate_results[0].missing_required_evidence,
        vec!["authority"]
    );
    Ok(())
}

#[test]
fn observation_journal_records_run_and_step_cost() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("observation.cost")?;
    let step_id = StepId::new("validate")?;

    seed_step(&ledger, &run_id, &step_id)?;
    record_cost_observation(
        &ledger,
        CostObservationJournalRecord::run(run_id.clone(), cost_observation(3), 14),
    )?;
    record_cost_observation(
        &ledger,
        CostObservationJournalRecord::step(
            run_id.clone(),
            step_id.clone(),
            cost_observation(5),
            15,
        ),
    )?;

    let view = ledger.load_run_view(&run_id)?;
    assert_eq!(view.cost_observations.len(), 1);
    let Some(step) = view.steps.get(&step_id) else {
        panic!("expected validate step");
    };
    assert_eq!(step.cost_observations.len(), 1);
    assert_eq!(view.total_cost_usd_micros(), 8);
    Ok(())
}

fn seed_step(
    ledger: &InMemoryControlLedger,
    run_id: &RunId,
    step_id: &StepId,
) -> Result<(), Box<dyn Error>> {
    record_run_created(
        ledger,
        RunCreatedJournalRecord::new(run_id.clone(), "observe workflow", 10),
    )?;
    record_step_created(
        ledger,
        StepCreatedJournalRecord::new(run_id.clone(), step_id.clone(), "Validate", 11),
    )?;
    Ok(())
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

fn cost_observation(cost_usd_micros: u64) -> CostObservation {
    CostObservation {
        provider: "local".to_owned(),
        cost_usd_micros,
        ..CostObservation::default()
    }
}
