use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_run_activity, append_control_run_with_step_activity, must_ok,
};
use tempfile::TempDir;

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
