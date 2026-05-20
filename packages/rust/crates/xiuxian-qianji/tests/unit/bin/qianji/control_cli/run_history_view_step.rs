use super::support::{
    append_control_run_with_run_activity, append_control_run_with_step,
    append_control_run_with_step_activity, append_empty_control_run, must_ok,
};
use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
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

#[test]
fn run_control_activity_renders_run_scope_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_activity(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Activity {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: None,
            activity_id: "activity-run-search".to_string(),
            json: true,
        }),
        "control activity json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity output should be valid json",
    );

    assert_eq!(json["activity_id"], "activity-run-search");
    assert_eq!(json["status"], "completed");
    assert_eq!(json["task"]["activity_type"], "wendao.search");
    assert_eq!(json["worker_id"], "worker-search");
    assert_eq!(json["attempt"], 1);
    assert_eq!(json["result"]["output_hash"], "sha256:run-search-output");
    Ok(())
}

#[test]
fn run_control_activity_renders_step_scope_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_activity(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Activity {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            activity_id: "activity-step-llm".to_string(),
            json: false,
        }),
        "control activity text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Activity"));
    assert!(output.rendered.contains("- Activity: `activity-step-llm`"));
    assert!(output.rendered.contains("- Status: `failed`"));
    assert!(output.rendered.contains("- Activity type: `llm.plan`"));
    assert!(output.rendered.contains("- Task queue: `llm.openai`"));
    assert!(output.rendered.contains("- Attempt: `2`"));
    assert!(output.rendered.contains("- Failure code: `rate_limited`"));
    assert!(output.rendered.contains("- Failure retryable: `true`"));
    Ok(())
}

#[test]
fn run_control_activity_rejects_missing_activity() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_run_activity(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Activity {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: None,
        activity_id: "missing-activity".to_string(),
        json: false,
    }) else {
        return Err("missing activity should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("could not find activity `missing-activity`")
    );
    Ok(())
}
