use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, DuckDbControlLedger, RunId, StepId, StepStatus,
};

#[test]
fn duckdb_ledger_replays_run_after_fresh_connection() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;
    let database_path = temp_dir.path().join("control-ledger.duckdb");
    let run_id = RunId::new("run-duckdb-replay")?;
    let step_id = StepId::new("step-plan")?;

    {
        let ledger = DuckDbControlLedger::open(&database_path)?;
        let first = ledger.append_event(ControlEvent::run(
            run_id.clone(),
            1,
            ControlEventKind::RunCreated {
                intent: "persist control events".to_owned(),
                budget: None,
                metadata: serde_json::Value::Null,
            },
        ))?;
        let second = ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id.clone(),
            2,
            ControlEventKind::StepCreated {
                title: "Persist plan".to_owned(),
                required_evidence: vec!["durable_ledger".to_owned()],
                budget: None,
            },
        ))?;

        assert_eq!(first.sequence, 1);
        assert_eq!(second.sequence, 2);
    }

    let reopened = DuckDbControlLedger::open(database_path)?;
    let events = reopened.load_events(&run_id)?;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].sequence, 1);
    assert_eq!(events[1].sequence, 2);

    let view = reopened.load_run_view(&run_id)?;
    assert_eq!(view.intent, Some("persist control events".to_owned()));
    let step = view
        .steps
        .get(&step_id)
        .ok_or_else(|| io::Error::other("missing replayed DuckDB step"))?;
    assert_eq!(step.status, StepStatus::Pending);
    assert_eq!(step.required_evidence, vec!["durable_ledger".to_owned()]);
    Ok(())
}

#[test]
fn duckdb_ledger_filters_runs_without_resetting_global_sequence() -> Result<(), Box<dyn Error>> {
    let ledger = DuckDbControlLedger::open_in_memory()?;
    let run_a = RunId::new("run-a")?;
    let run_b = RunId::new("run-b")?;

    let first = ledger.append_event(run_created(run_a.clone(), "a"))?;
    let second = ledger.append_event(run_created(run_b.clone(), "b"))?;
    let third = ledger.append_event(ControlEvent::run(
        run_a.clone(),
        3,
        ControlEventKind::RunAdmitted,
    ))?;

    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);
    assert_eq!(third.sequence, 3);

    let run_a_events = ledger.load_events(&run_a)?;
    assert_eq!(run_a_events.len(), 2);
    assert_eq!(run_a_events[0].sequence, 1);
    assert_eq!(run_a_events[1].sequence, 3);
    Ok(())
}

fn run_created(run_id: RunId, intent: &str) -> ControlEvent {
    ControlEvent::run(
        run_id,
        1,
        ControlEventKind::RunCreated {
            intent: intent.to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    )
}
