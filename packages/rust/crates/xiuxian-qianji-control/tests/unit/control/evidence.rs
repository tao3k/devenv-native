use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, EvidenceGate, EvidenceId, EvidenceRef,
    InMemoryControlLedger, RequiredEvidenceGate, RunId, StepId,
};

#[test]
fn in_memory_ledger_replays_required_evidence_gate() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-required-evidence")?;
    let step_id = StepId::new("validate-frontier")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "validate required evidence coverage".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Validate frontier".to_owned(),
            required_evidence: vec![
                "ownership_boundary".to_owned(),
                "validation_path".to_owned(),
            ],
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        3,
        ControlEventKind::EvidenceAttached {
            evidence: EvidenceRef {
                evidence_id: EvidenceId::new("evidence-validation-path")?,
                requirement_key: Some("validation_path".to_owned()),
                source: "unit-test".to_owned(),
                uri: None,
                summary: None,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;

    let view = ledger.load_run_view(&run_id)?;
    let step = view
        .steps
        .get(&step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;
    let gate = RequiredEvidenceGate::new("required-evidence")?;
    let result = gate.evaluate(step);

    assert!(!result.passed);
    assert_eq!(
        result.selected_required_evidence,
        vec!["validation_path".to_owned()]
    );
    assert_eq!(
        result.missing_required_evidence,
        vec!["ownership_boundary".to_owned()]
    );

    Ok(())
}
