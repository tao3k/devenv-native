use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_run_timer, append_control_run_with_step_timer, must_ok,
};
use tempfile::TempDir;

#[test]
fn run_control_timer_renders_run_scope_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_timer(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Timer {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: None,
            timer_id: "timer-run-wakeup".to_string(),
            json: true,
        }),
        "control timer json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "timer output should be valid json",
    );

    assert_eq!(json["timer_id"], "timer-run-wakeup");
    assert_eq!(json["status"], "fired");
    assert_eq!(json["timer"]["fire_at_ms"], 10_000);
    assert_eq!(json["fired_at_ms"], 10_250);
    Ok(())
}

#[test]
fn run_control_timer_renders_step_scope_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_timer(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Timer {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            timer_id: "timer-step-approval-timeout".to_string(),
            json: false,
        }),
        "control timer text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Timer"));
    assert!(
        output
            .rendered
            .contains("- Timer: `timer-step-approval-timeout`")
    );
    assert!(output.rendered.contains("- Status: `scheduled`"));
    assert!(output.rendered.contains("- Fire at ms: `20000`"));
    assert!(output.rendered.contains("- Fired at ms: `<none>`"));
    Ok(())
}

#[test]
fn run_control_timer_rejects_missing_timer() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_timer(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Timer {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: None,
        timer_id: "missing-timer".to_string(),
        json: false,
    }) else {
        return Err("missing timer should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("could not find timer `missing-timer`")
    );
    Ok(())
}
