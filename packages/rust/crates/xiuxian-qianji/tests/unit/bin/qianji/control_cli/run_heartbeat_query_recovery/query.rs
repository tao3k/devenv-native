use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_step_signal_and_timer, append_control_run_with_step_timer, must_ok,
};
use tempfile::TempDir;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger};

#[test]
fn run_control_query_state_renders_json_without_appending() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_timer(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let before_count = must_ok(
        ledger.load_events(&run_id),
        "should load events before state query",
    )
    .len();

    let output = must_ok(
        run_control_command(&ControlCliCommand::QueryState {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            now_ms: 9_000,
            json: true,
        }),
        "control query state json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "state query output should be valid json",
    );
    let after_count = must_ok(
        ledger.load_events(&run_id),
        "should load events after state query",
    )
    .len();

    assert_eq!(json["event_count"], before_count);
    assert_eq!(json["run_view"]["run_id"], "run-control-cli");
    assert_eq!(
        json["run_view"]["steps"]["run-control-step"]["timers"]["timer-step-approval-timeout"]["status"],
        "scheduled"
    );
    assert_eq!(json["recovery_snapshot"]["observed_at_ms"], 9_000);
    assert_eq!(before_count, after_count);
    Ok(())
}

#[test]
fn run_control_query_state_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_signal_and_timer(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::QueryState {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 30_000,
            json: false,
        }),
        "control query state text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control State"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Status: `draft`"));
    assert!(output.rendered.contains("- Steps: `1`"));
    assert!(output.rendered.contains("- Step timers: `1`"));
    assert!(output.rendered.contains("- Step signals: `1`"));
    assert!(output.rendered.contains("- Fireable timers: `1`"));
    Ok(())
}
