use std::error::Error;

use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, CostObservation, InMemoryControlLedger,
    RecoveryItemScope, RunId, StepId,
};

#[test]
fn cost_inventory_projection_surfaces_run_and_step_observations() -> Result<(), Box<dyn Error>> {
    let ledger = cost_inventory_fixture()?;
    let run_id = RunId::new("run-cost-inventory")?;
    let projection = ledger.load_cost_inventory_projection(&run_id)?;

    assert_eq!(projection.run_id, run_id);
    assert_eq!(projection.items.len(), 2);
    assert_eq!(projection.summary.total, 2);
    assert_eq!(projection.summary.run_scoped, 1);
    assert_eq!(projection.summary.step_scoped, 1);
    assert_eq!(projection.summary.total_tokens, 42);
    assert_eq!(projection.summary.cost_usd_micros, 130);
    assert_eq!(projection.summary.latency_ms, 1_250);
    assert_eq!(projection.summary.latency_observations, 2);
    assert_eq!(projection.items[0].sequence, 2);
    assert_eq!(projection.items[0].scope, RecoveryItemScope::run());
    assert_eq!(
        projection.items[1].scope,
        RecoveryItemScope::step(StepId::new("step-cost-inventory")?)
    );
    assert_eq!(projection.items[1].observation.provider, "tool.github");
    Ok(())
}

fn cost_inventory_fixture() -> Result<InMemoryControlLedger, Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-cost-inventory")?;
    let step_id = StepId::new("step-cost-inventory")?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "cost inventory projection".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        10_000,
        ControlEventKind::CostObserved {
            observation: cost_observation("llm.openai", Some("gpt-test"), 10, 20, 100, 1_000),
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        11_000,
        ControlEventKind::StepCreated {
            title: "Run tool".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id,
        step_id,
        20_000,
        ControlEventKind::CostObserved {
            observation: cost_observation("tool.github", None, 5, 7, 30, 250),
        },
    ))?;
    Ok(ledger)
}

fn cost_observation(
    provider: &str,
    model: Option<&str>,
    prompt_tokens: u64,
    completion_tokens: u64,
    cost_usd_micros: u64,
    latency_ms: u64,
) -> CostObservation {
    CostObservation {
        provider: provider.to_owned(),
        model: model.map(str::to_owned),
        prompt_tokens,
        completion_tokens,
        total_tokens: None,
        cost_usd_micros,
        latency_ms: Some(latency_ms),
    }
}
