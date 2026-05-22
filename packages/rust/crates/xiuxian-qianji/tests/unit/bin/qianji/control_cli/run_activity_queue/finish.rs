use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, must_ok,
};
use tempfile::TempDir;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger};

#[test]
fn run_control_activity_complete_appends_json_and_is_idempotent() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let command = ControlCliCommand::ActivityComplete {
        ledger_path: ledger_path.clone(),
        run_id: run_id.as_str().to_string(),
        step_id: None,
        activity_id: "activity-run-started".to_string(),
        completed_at_ms: 9_000,
        output_hash: Some("sha256:activity-output".to_string()),
        metadata: Some("{\"rows\":3}".to_string()),
        json: true,
    };

    let first = must_ok(
        run_control_command(&command),
        "first activity-complete should append",
    );
    let second = must_ok(
        run_control_command(&command),
        "duplicate activity-complete should be idempotent",
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
        first_json["record"]["event"]["kind"]["event"],
        "activity_completed"
    );
    assert_eq!(
        first_json["record"]["event"]["kind"]["result"]["output_hash"],
        "sha256:activity-output"
    );
    assert_eq!(
        first_json["record"]["sequence"],
        second_json["record"]["sequence"]
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "activity complete should replay into queue projection summary",
    );
    assert_eq!(queue.items.len(), 2);
    assert_eq!(queue.summary.total, 3);
    assert_eq!(queue.summary.scheduled, 2);
    assert_eq!(queue.summary.in_flight, 0);
    assert_eq!(queue.summary.completed, 1);
    assert_eq!(queue.summary.failed, 0);
    Ok(())
}

#[test]
fn run_control_activity_complete_accepts_worker_task_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let worker_task_json = worker_task_json(&ledger_path, &run_id, "activity-run-scheduled")?;
    must_ok(
        run_control_command(&ControlCliCommand::ActivityStartWorkerTask {
            ledger_path: ledger_path.clone(),
            worker_task_json: worker_task_json.clone(),
            worker_id: "worker-llm".to_string(),
            started_at_ms: 8_000,
            json: false,
        }),
        "activity-complete worker-task setup should start the run activity",
    );

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityCompleteWorkerTask {
            ledger_path: ledger_path.clone(),
            worker_task_json,
            completed_at_ms: 9_000,
            output_hash: Some("sha256:activity-output".to_string()),
            metadata: Some("{\"rows\":3}".to_string()),
            json: true,
        }),
        "control activity-complete worker-task json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity-complete worker-task output should be valid json",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker-task activity complete should replay into queue projection summary",
    );

    assert_eq!(json["status"], "appended");
    assert_eq!(
        json["record"]["event"]["kind"]["event"],
        "activity_completed"
    );
    assert_eq!(
        json["record"]["event"]["kind"]["activity_id"],
        "activity-run-scheduled"
    );
    assert_eq!(queue.summary.completed, 1);
    assert_eq!(queue.summary.scheduled, 1);
    Ok(())
}

#[test]
fn run_control_activity_fail_appends_json_after_start() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    must_ok(
        run_control_command(&ControlCliCommand::ActivityStart {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            activity_id: "activity-step-scheduled".to_string(),
            worker_id: "worker-tool".to_string(),
            started_at_ms: 8_000,
            attempt: 1,
            json: false,
        }),
        "activity-fail setup should start the step activity",
    );

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityFail {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            activity_id: "activity-step-scheduled".to_string(),
            failed_at_ms: 9_000,
            error_code: "rate_limited".to_string(),
            message: "provider rejected request".to_string(),
            retryable: true,
            attempt: 1,
            metadata: None,
            json: true,
        }),
        "control activity-fail json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity-fail output should be valid json",
    );

    assert_eq!(json["status"], "appended");
    assert_eq!(json["record"]["event"]["kind"]["event"], "activity_failed");
    assert_eq!(
        json["record"]["event"]["kind"]["failure"]["error_code"],
        "rate_limited"
    );
    assert_eq!(
        json["record"]["event"]["kind"]["failure"]["retryable"],
        true
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "activity failure should replay into queue projection summary",
    );
    assert_eq!(queue.items.len(), 1);
    assert_eq!(queue.summary.total, 3);
    assert_eq!(queue.summary.scheduled, 1);
    assert_eq!(queue.summary.in_flight, 1);
    assert_eq!(queue.summary.completed, 0);
    assert_eq!(queue.summary.failed, 1);
    Ok(())
}

#[test]
fn run_control_activity_fail_accepts_worker_task_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let worker_task_json = worker_task_json(&ledger_path, &run_id, "activity-step-scheduled")?;
    must_ok(
        run_control_command(&ControlCliCommand::ActivityStartWorkerTask {
            ledger_path: ledger_path.clone(),
            worker_task_json: worker_task_json.clone(),
            worker_id: "worker-tool".to_string(),
            started_at_ms: 8_000,
            json: false,
        }),
        "activity-fail worker-task setup should start the step activity",
    );

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityFailWorkerTask {
            ledger_path: ledger_path.clone(),
            worker_task_json,
            failed_at_ms: 9_000,
            error_code: "rate_limited".to_string(),
            message: "provider rejected request".to_string(),
            retryable: true,
            metadata: None,
            json: true,
        }),
        "control activity-fail worker-task json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity-fail worker-task output should be valid json",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "worker-task activity failure should replay into queue projection summary",
    );

    assert_eq!(json["status"], "appended");
    assert_eq!(json["record"]["event"]["kind"]["event"], "activity_failed");
    assert_eq!(
        json["record"]["event"]["kind"]["activity_id"],
        "activity-step-scheduled"
    );
    assert_eq!(json["record"]["event"]["kind"]["failure"]["attempt"], 1);
    assert_eq!(queue.summary.failed, 1);
    assert_eq!(queue.summary.scheduled, 1);
    Ok(())
}

#[test]
fn run_control_activity_complete_rejects_not_started_activity() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::ActivityComplete {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: None,
        activity_id: "activity-run-scheduled".to_string(),
        completed_at_ms: 9_000,
        output_hash: None,
        metadata: None,
        json: false,
    }) else {
        return Err("activity complete before start should fail".to_string());
    };

    assert!(
        error
            .to_string()
            .contains("activity completion requires a started activity")
    );
    Ok(())
}

fn worker_task_json(
    ledger_path: &std::path::Path,
    run_id: &xiuxian_qianji_control::RunId,
    activity_id: &str,
) -> Result<String, String> {
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let task = must_ok(
        ledger.load_worker_activity_tasks(run_id, None),
        "should load worker activity tasks",
    )
    .into_iter()
    .find(|task| task.activity_id.as_str() == activity_id)
    .ok_or_else(|| format!("missing worker task for {activity_id}"))?;
    serde_json::to_string(&task).map_err(|error| format!("should serialize worker task: {error}"))
}
