use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_run_and_step_signals, append_control_run_with_step,
    append_empty_control_run, must_ok, must_some,
};
use tempfile::TempDir;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, SignalName, StepId};

#[test]
fn run_control_signal_appends_run_scope_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Signal {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            step_id: None,
            signal_name: "run.refresh".to_string(),
            payload: r#"{"reason":"manual"}"#.to_string(),
            received_at_ms: 11_000,
            json: true,
        }),
        "control signal json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "signal output should be valid json",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let view = must_ok(
        ledger.load_run_view(&run_id),
        "signal append should replay into run view",
    );

    assert_eq!(json["sequence"], 2);
    assert_eq!(json["event"]["run_id"], "run-control-cli");
    assert_eq!(json["event"]["kind"]["event"], "signal_received");
    assert_eq!(
        json["event"]["kind"]["signal"]["signal_name"],
        "run.refresh"
    );
    assert_eq!(
        json["event"]["kind"]["signal"]["metadata"]["reason"],
        "manual"
    );
    assert_eq!(view.signals.len(), 1);
    assert_eq!(
        view.signals[0].signal_name,
        must_ok(SignalName::new("run.refresh"), "should build signal name")
    );
    Ok(())
}

#[test]
fn run_control_signal_appends_step_scope_text_and_replays() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Signal {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            signal_name: "human.approval".to_string(),
            payload: r#"{"approved":true,"approver":"operator"}"#.to_string(),
            received_at_ms: 12_000,
            json: false,
        }),
        "control signal text should render",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let view = must_ok(
        ledger.load_run_view(&run_id),
        "signal append should replay into run view",
    );
    let step = must_some(
        view.steps.get(&must_ok(
            StepId::new("run-control-step"),
            "should build step id",
        )),
        "step signal should replay into step view",
    );

    assert!(output.rendered.starts_with("# Qianji Control Signal"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Scope: `run-control-step`"));
    assert!(output.rendered.contains("- Signal: `human.approval`"));
    assert!(output.rendered.contains("- Received at ms: `12000`"));
    assert_eq!(step.signals.len(), 1);
    assert_eq!(
        step.signals[0].signal_name,
        must_ok(
            SignalName::new("human.approval"),
            "should build signal name"
        )
    );
    assert_eq!(step.signals[0].metadata["approved"], true);
    Ok(())
}

#[test]
fn run_control_signal_rejects_invalid_payload_without_append() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Signal {
        ledger_path: ledger_path.clone(),
        run_id: run_id.as_str().to_string(),
        step_id: None,
        signal_name: "run.refresh".to_string(),
        payload: "{not-json".to_string(),
        received_at_ms: 11_000,
        json: false,
    }) else {
        return Err("invalid signal payload should fail".to_string());
    };
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let records = must_ok(
        ledger.load_events(&run_id),
        "invalid payload should not append an event",
    );

    assert!(
        error
            .to_string()
            .contains("invalid `--payload` JSON for `control signal`")
    );
    assert_eq!(records.len(), 1);
    Ok(())
}

#[test]
fn run_control_signals_renders_json_inventory() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_and_step_signals(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Signals {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: true,
        }),
        "control signals json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "signals output should be valid json",
    );
    let items = json["items"]
        .as_array()
        .ok_or_else(|| "signals output should include item array".to_string())?;

    assert_eq!(json["summary"]["total"], 2);
    assert_eq!(json["summary"]["run_scoped"], 1);
    assert_eq!(json["summary"]["step_scoped"], 1);
    assert_eq!(items[0]["signal"]["signal_name"], "run.refresh");
    assert_eq!(items[0]["scope"]["scope"], "run");
    assert_eq!(items[1]["signal"]["signal_name"], "human.approval");
    assert_eq!(items[1]["scope"]["scope"], "step");
    assert_eq!(items[1]["scope"]["step_id"], "run-control-step");
    assert_eq!(items[1]["received_at_ms"], 12_000);
    Ok(())
}

#[test]
fn run_control_signals_renders_empty_text_inventory() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Signals {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: false,
        }),
        "control signals text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Signals"));
    assert!(
        output
            .rendered
            .contains("- Signals: total `0`, run `0`, step `0`")
    );
    Ok(())
}
