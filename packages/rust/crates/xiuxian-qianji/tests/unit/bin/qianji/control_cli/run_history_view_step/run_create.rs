use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::must_ok;
use tempfile::TempDir;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

#[test]
fn run_control_run_create_appends_run_created_event() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");

    let output = must_ok(
        run_control_command(&ControlCliCommand::RunCreate {
            ledger_path: ledger_path.clone(),
            run_id: "run-control-cli".to_string(),
            occurred_at_ms: 42,
            intent: "admit schedule plan".to_string(),
            json: true,
        }),
        "control run-create json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "run-create output should be valid json",
    );

    assert_eq!(json["sequence"], 1);
    assert_eq!(json["event"]["run_id"], "run-control-cli");
    assert_eq!(json["event"]["kind"]["event"], "run_created");
    assert_eq!(json["event"]["kind"]["intent"], "admit schedule plan");

    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let run_id = must_ok(RunId::new("run-control-cli"), "should build run id");
    let records = must_ok(
        ledger.load_events(&run_id),
        "should load durable run-create event",
    );
    assert_eq!(records.len(), 1);
    Ok(())
}

#[test]
fn run_control_run_create_renders_text() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");

    let output = must_ok(
        run_control_command(&ControlCliCommand::RunCreate {
            ledger_path,
            run_id: "run-control-cli".to_string(),
            occurred_at_ms: 42,
            intent: "admit schedule plan".to_string(),
            json: false,
        }),
        "control run-create text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Run Create"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Sequence: `1`"));
    Ok(())
}
