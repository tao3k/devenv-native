use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, append_empty_control_run, must_ok,
};
use tempfile::TempDir;
use xiuxian_qianji_control::{ActivityId, ControlLedger, DuckDbControlLedger};

#[test]
fn run_control_activity_start_appends_json_and_updates_queue() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityStart {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            step_id: None,
            activity_id: "activity-run-scheduled".to_string(),
            worker_id: "worker-llm".to_string(),
            started_at_ms: 7_000,
            attempt: 1,
            json: true,
        }),
        "control activity-start json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity-start output should be valid json",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "activity start should replay into queue projection",
    );

    assert_eq!(json["status"], "appended");
    assert_eq!(json["record"]["event"]["kind"]["event"], "activity_started");
    assert_eq!(
        json["record"]["event"]["kind"]["activity_id"],
        "activity-run-scheduled"
    );
    assert_eq!(json["record"]["event"]["kind"]["worker_id"], "worker-llm");
    assert_eq!(queue.items.len(), 1);
    assert_eq!(queue.summary.total, 3);
    assert_eq!(queue.summary.scheduled, 1);
    assert_eq!(queue.summary.in_flight, 2);
    assert_eq!(queue.summary.completed, 0);
    assert_eq!(queue.summary.failed, 0);
    assert_eq!(
        queue.items[0].activity.activity_id,
        must_ok(
            ActivityId::new("activity-step-scheduled"),
            "should build queued activity id"
        )
    );
    Ok(())
}

#[test]
fn run_control_activity_start_is_idempotent_for_exact_duplicate() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let command = ControlCliCommand::ActivityStart {
        ledger_path: ledger_path.clone(),
        run_id: run_id.as_str().to_string(),
        step_id: Some("run-control-step".to_string()),
        activity_id: "activity-step-scheduled".to_string(),
        worker_id: "worker-tool".to_string(),
        started_at_ms: 8_000,
        attempt: 1,
        json: true,
    };

    let first = must_ok(
        run_control_command(&command),
        "first activity-start should append",
    );
    let second = must_ok(
        run_control_command(&command),
        "duplicate activity-start should be idempotent",
    );
    let first_json: serde_json::Value = must_ok(
        serde_json::from_str(&first.rendered),
        "first output should be valid json",
    );
    let second_json: serde_json::Value = must_ok(
        serde_json::from_str(&second.rendered),
        "second output should be valid json",
    );

    assert_eq!(first_json["status"], "appended");
    assert_eq!(second_json["status"], "already_recorded");
    assert_eq!(
        first_json["record"]["sequence"],
        second_json["record"]["sequence"]
    );
    Ok(())
}

#[test]
fn run_control_activity_start_rejects_unscheduled_activity() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::ActivityStart {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: None,
        activity_id: "missing-activity".to_string(),
        worker_id: "worker-llm".to_string(),
        started_at_ms: 7_000,
        attempt: 1,
        json: false,
    }) else {
        return Err("unscheduled activity start should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("activity start requires a scheduled activity")
    );
    Ok(())
}
