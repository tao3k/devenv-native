use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_active_step_lease, append_control_run_with_step, must_ok,
};
use tempfile::TempDir;

#[test]
fn run_control_lease_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_active_step_lease(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Lease {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: "run-control-step".to_string(),
            json: true,
        }),
        "control lease json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "lease output should be valid json",
    );

    assert_eq!(json["lease_id"], "lease-control-step");
    assert_eq!(json["run_id"], "run-control-cli");
    assert_eq!(json["step_id"], "run-control-step");
    assert_eq!(json["worker_id"], "worker-control");
    assert_eq!(json["acquired_at_ms"], 10_000);
    assert_eq!(json["expires_at_ms"], 20_000);
    Ok(())
}

#[test]
fn run_control_lease_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_active_step_lease(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Lease {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: "run-control-step".to_string(),
            json: false,
        }),
        "control lease text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Lease"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Step: `run-control-step`"));
    assert!(output.rendered.contains("- Lease: `lease-control-step`"));
    assert!(output.rendered.contains("- Worker: `worker-control`"));
    Ok(())
}

#[test]
fn run_control_lease_rejects_step_without_active_lease() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Lease {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: "run-control-step".to_string(),
        json: false,
    }) else {
        return Err("step without active lease should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("step `run-control-step` in run `run-control-cli` has no active lease")
    );
    Ok(())
}
