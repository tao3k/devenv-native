use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{append_empty_control_run, must_ok};
use tempfile::TempDir;

#[test]
fn run_control_history_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::History {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: true,
        }),
        "control history json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "history output should be valid json",
    );

    assert_eq!(json.as_array().map(Vec::len), Some(1));
    assert_eq!(json[0]["sequence"], 1);
    assert_eq!(json[0]["event"]["run_id"], "run-control-cli");
    assert_eq!(json[0]["event"]["kind"]["event"], "run_created");
    Ok(())
}

#[test]
fn run_control_history_renders_text_timeline() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::History {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: false,
        }),
        "control history text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control History"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Events: `1`"));
    assert!(output.rendered.contains("- #1 @1 [run] `run_created`"));
    Ok(())
}
