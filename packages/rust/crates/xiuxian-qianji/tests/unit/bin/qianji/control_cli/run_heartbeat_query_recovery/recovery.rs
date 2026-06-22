use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{append_empty_control_run, must_ok};
use tempfile::TempDir;

#[test]
fn run_control_recovery_snapshot_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::RecoverySnapshot {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 9_000,
            json: true,
        }),
        "control recovery snapshot json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "snapshot output should be valid json",
    );

    assert_eq!(json["run_id"], "run-control-cli");
    assert_eq!(json["observed_at_ms"], 9_000);
    assert_eq!(json["summary"]["total_actions"], 0);
    assert_eq!(json["plan"]["actions"].as_array().map(Vec::len), Some(0));
    Ok(())
}

#[test]
fn run_control_recovery_snapshot_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::RecoverySnapshot {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 9_000,
            json: false,
        }),
        "control recovery snapshot text should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Control Recovery Snapshot")
    );
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Total actions: `0`"));
    Ok(())
}
