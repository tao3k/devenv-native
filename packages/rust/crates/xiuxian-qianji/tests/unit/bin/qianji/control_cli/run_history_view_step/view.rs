use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{append_control_run_with_step, must_ok};
use tempfile::TempDir;

#[test]
fn run_control_view_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::View {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: true,
        }),
        "control view json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "view output should be valid json",
    );

    assert_eq!(json["run_id"], "run-control-cli");
    assert_eq!(json["status"], "draft");
    assert_eq!(json["steps"].as_object().map(serde_json::Map::len), Some(1));
    assert_eq!(
        json["steps"]["run-control-step"]["title"],
        "Review durable state"
    );
    Ok(())
}

#[test]
fn run_control_view_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::View {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: false,
        }),
        "control view text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control View"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Status: `draft`"));
    assert!(output.rendered.contains("- Steps: `1`"));
    assert!(
        output
            .rendered
            .contains("`run-control-step` [pending] Review durable state")
    );
    Ok(())
}
