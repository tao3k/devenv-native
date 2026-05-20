use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_run_and_step_costs, append_empty_control_run, must_ok,
};
use tempfile::TempDir;

#[test]
fn run_control_costs_renders_json_inventory() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_and_step_costs(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Costs {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: true,
        }),
        "control costs json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "costs output should be valid json",
    );
    let items = json["items"]
        .as_array()
        .ok_or_else(|| "costs output should include item array".to_string())?;

    assert_eq!(json["summary"]["total"], 2);
    assert_eq!(json["summary"]["run_scoped"], 1);
    assert_eq!(json["summary"]["step_scoped"], 1);
    assert_eq!(json["summary"]["total_tokens"], 42);
    assert_eq!(json["summary"]["cost_usd_micros"], 130);
    assert_eq!(items[0]["observation"]["provider"], "llm.openai");
    assert_eq!(items[0]["scope"]["scope"], "run");
    assert_eq!(items[1]["observation"]["provider"], "tool.github");
    assert_eq!(items[1]["scope"]["scope"], "step");
    assert_eq!(items[1]["scope"]["step_id"], "run-control-step");
    Ok(())
}

#[test]
fn run_control_costs_renders_empty_text_inventory() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Costs {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            json: false,
        }),
        "control costs text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Costs"));
    assert!(
        output
            .rendered
            .contains("- Observations: total `0`, run `0`, step `0`")
    );
    assert!(output.rendered.contains("- Cost usd micros: `0`"));
    Ok(())
}
