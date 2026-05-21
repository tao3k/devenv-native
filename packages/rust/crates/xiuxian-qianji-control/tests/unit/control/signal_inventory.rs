use std::error::Error;

use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, InMemoryControlLedger, RecoveryItemScope, RunId,
    SignalName, SignalRecord, StepId,
};

#[test]
fn signal_inventory_projection_surfaces_run_and_step_signals() -> Result<(), Box<dyn Error>> {
    let ledger = signal_inventory_fixture()?;
    let run_id = RunId::new("run-signal-inventory")?;
    let projection = ledger.load_signal_inventory_projection(&run_id)?;

    assert_eq!(projection.run_id, run_id);
    assert_eq!(projection.items.len(), 2);
    assert_eq!(projection.summary.total, 2);
    assert_eq!(projection.summary.run_scoped, 1);
    assert_eq!(projection.summary.step_scoped, 1);
    assert_eq!(projection.items[0].sequence, 2);
    assert_eq!(
        projection.items[0].signal.signal_name,
        SignalName::new("system.ready")?
    );
    assert_eq!(projection.items[0].scope, RecoveryItemScope::run());
    assert_eq!(projection.items[1].received_at_ms, 20_000);
    assert_eq!(
        projection.items[1].scope,
        RecoveryItemScope::step(StepId::new("step-signal-inventory")?)
    );
    assert_eq!(projection.items[1].signal.metadata["approved"], true);
    Ok(())
}

fn signal_inventory_fixture() -> Result<InMemoryControlLedger, Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-signal-inventory")?;
    let step_id = StepId::new("step-signal-inventory")?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "signal inventory projection".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        10_000,
        ControlEventKind::SignalReceived {
            signal: signal_record("system.ready", serde_json::json!({"ready": true}))?,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        11_000,
        ControlEventKind::StepCreated {
            title: "Await approval".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id,
        step_id,
        20_000,
        ControlEventKind::SignalReceived {
            signal: signal_record("human.approval", serde_json::json!({"approved": true}))?,
        },
    ))?;
    Ok(ledger)
}

fn signal_record(name: &str, metadata: serde_json::Value) -> Result<SignalRecord, Box<dyn Error>> {
    Ok(SignalRecord {
        signal_name: SignalName::new(name)?,
        payload_ref: None,
        payload_hash: None,
        metadata,
    })
}
