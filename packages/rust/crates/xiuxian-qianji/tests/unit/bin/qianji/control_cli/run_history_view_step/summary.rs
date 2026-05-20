use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_operator_summary_facts, append_empty_control_run, must_ok,
};
use tempfile::TempDir;

#[test]
fn run_control_summary_renders_json_operator_view() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_operator_summary_facts(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Summary {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 25_000,
            json: true,
        }),
        "control summary json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "summary output should be valid json",
    );

    assert_eq!(json["run_id"], "run-control-cli");
    assert_eq!(json["event_count"], 7);
    assert_eq!(json["steps"], 1);
    assert_eq!(json["active_leases"], 1);
    assert_eq!(json["activities"]["scheduled"], 1);
    assert_eq!(json["timers"]["scheduled"], 1);
    assert_eq!(json["signals"]["step_scoped"], 1);
    assert_eq!(json["costs"]["cost_usd_micros"], 100);
    assert_eq!(json["recovery"]["reclaim_expired_leases"], 1);
    assert_eq!(json["recovery"]["fireable_timers"], 1);
    Ok(())
}

#[test]
fn run_control_summary_renders_empty_text_operator_view() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Summary {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 25_000,
            json: false,
        }),
        "control summary text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Summary"));
    assert!(output.rendered.contains("- Events: `1`"));
    assert!(output.rendered.contains("- Steps: `0`"));
    assert!(output.rendered.contains("- Recovery actions: `0`"));
    Ok(())
}
