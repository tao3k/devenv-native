use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_active_step_lease, append_empty_control_run, must_ok,
};
use tempfile::TempDir;

#[test]
fn run_control_leases_renders_json_inventory() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_active_step_lease(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Leases {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: true,
        }),
        "control leases json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "leases output should be valid json",
    );
    let leases = json
        .as_array()
        .ok_or_else(|| "leases output should be a json array".to_string())?;

    assert_eq!(leases.len(), 1);
    assert_eq!(leases[0]["lease_id"], "lease-control-step");
    assert_eq!(leases[0]["step_id"], "run-control-step");
    assert_eq!(leases[0]["worker_id"], "worker-control");
    Ok(())
}

#[test]
fn run_control_leases_renders_empty_text_inventory() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Leases {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: false,
        }),
        "control leases text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Leases"));
    assert!(output.rendered.contains("Active leases: `0`"));
    Ok(())
}
