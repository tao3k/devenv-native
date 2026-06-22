use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_llm_activity_inventory, append_empty_control_run, must_ok,
};
use tempfile::TempDir;

#[test]
fn run_control_llm_activities_renders_json_inventory() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_llm_activity_inventory(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::LlmActivities {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            require_request_audit: false,
            json: true,
        }),
        "control llm-activities json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "llm-activities output should be valid json",
    );
    let items = json["items"]
        .as_array()
        .ok_or_else(|| "llm-activities output should include item array".to_string())?;

    assert_eq!(json["summary"]["total"], 2);
    assert_eq!(json["summary"]["scheduled"], 1);
    assert_eq!(json["summary"]["completed"], 1);
    assert_eq!(json["summary"]["missing_request_audit"], 1);
    assert_eq!(items[0]["activity_id"], "activity-run-llm-plan");
    assert_eq!(items[0]["scope"]["scope"], "run");
    assert_eq!(items[0]["model"], "openai/gpt-5-mini");
    assert_eq!(items[1]["activity_id"], "activity-step-llm-repair");
    assert_eq!(items[1]["scope"]["scope"], "step");
    assert_eq!(items[1]["scope"]["step_id"], "run-control-step");
    assert!(items[1]["request_audit_metadata"].is_null());
    Ok(())
}

#[test]
fn run_control_llm_activities_renders_empty_text_inventory() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::LlmActivities {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            require_request_audit: false,
            json: false,
        }),
        "control llm-activities text should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Control LLM Activities")
    );
    assert!(output.rendered.contains(
        "- Activities: total `0`, scheduled `0`, in-flight `0`, completed `0`, failed `0`"
    ));
    assert!(output.rendered.contains("- Missing request audit: `0`"));
    Ok(())
}

#[test]
fn run_control_llm_activities_rejects_missing_request_audit_when_required() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_llm_activity_inventory(&ledger_path);

    let result = run_control_command(&ControlCliCommand::LlmActivities {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        require_request_audit: true,
        json: true,
    });
    let error = match result {
        Ok(value) => panic!("request-audit gate should reject incomplete LLM inventory: {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("found 1 LLM activity row(s) without request audit metadata")
    );
    Ok(())
}

#[test]
fn run_control_llm_activities_allows_empty_inventory_when_audit_required() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::LlmActivities {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            require_request_audit: true,
            json: false,
        }),
        "empty llm inventory should satisfy request-audit gate",
    );

    assert!(output.rendered.contains("- Missing request audit: `0`"));
    Ok(())
}
