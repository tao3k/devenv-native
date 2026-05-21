use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{append_control_run_with_step, must_ok};
use tempfile::TempDir;

#[test]
fn run_control_step_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Step {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: "run-control-step".to_string(),
            json: true,
        }),
        "control step json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "step output should be valid json",
    );

    assert_eq!(json["step_id"], "run-control-step");
    assert_eq!(json["status"], "pending");
    assert_eq!(json["title"], "Review durable state");
    assert_eq!(json["required_evidence"].as_array().map(Vec::len), Some(1));
    Ok(())
}

#[test]
fn run_control_step_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Step {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: "run-control-step".to_string(),
            json: false,
        }),
        "control step text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Step"));
    assert!(output.rendered.contains("- Step: `run-control-step`"));
    assert!(output.rendered.contains("- Status: `pending`"));
    assert!(output.rendered.contains("- Required evidence: `1`"));
    assert!(output.rendered.contains("- Covered evidence: `0`"));
    Ok(())
}

#[test]
fn run_control_step_rejects_missing_step() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Step {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: "missing-step".to_string(),
        json: false,
    }) else {
        return Err("missing step should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("could not find step `missing-step`")
    );
    Ok(())
}
