use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, must_ok,
};
use tempfile::TempDir;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger};

#[test]
fn run_control_activity_queue_renders_json_without_appending() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let before_count = must_ok(
        ledger.load_events(&run_id),
        "should load events before activity queue query",
    )
    .len();

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityQueue {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            task_queue: Some("llm.openai".to_string()),
            json: true,
        }),
        "control activity queue json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity queue output should be valid json",
    );
    let after_count = must_ok(
        ledger.load_events(&run_id),
        "should load events after activity queue query",
    )
    .len();

    assert_eq!(json["run_id"], "run-control-cli");
    assert_eq!(json["task_queue"], "llm.openai");
    assert_eq!(json["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(json["summary"]["total"], 2);
    assert_eq!(json["summary"]["scheduled"], 1);
    assert_eq!(json["summary"]["in_flight"], 1);
    assert_eq!(json["summary"]["completed"], 0);
    assert_eq!(json["summary"]["failed"], 0);
    assert_eq!(
        json["items"][0]["activity"]["activity_id"],
        "activity-run-scheduled"
    );
    assert_eq!(before_count, after_count);
    Ok(())
}

#[test]
fn run_control_activity_queue_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityQueue {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            task_queue: None,
            json: false,
        }),
        "control activity queue text should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Control Activity Queue")
    );
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Task queue: `<all>`"));
    assert!(output.rendered.contains("- Queue items: `2`"));
    assert!(output.rendered.contains(
        "- Activities: total `3`, scheduled `2`, in-flight `1`, completed `0`, failed `0`"
    ));
    assert!(output.rendered.contains("`activity-run-scheduled` [run]"));
    assert!(
        output
            .rendered
            .contains("`activity-step-scheduled` [step:run-control-step]")
    );
    Ok(())
}
