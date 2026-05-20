use super::support::{
    append_control_run_with_step_signal_and_timer, append_control_run_with_step_timer,
    append_empty_control_run, must_ok,
};
use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use tempfile::TempDir;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger};

#[test]
fn run_control_heartbeat_appends_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Heartbeat {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            worker_id: "worker-control".to_string(),
            observed_at_ms: 20_000,
            expires_at_ms: 35_000,
            metadata: Some(r#"{"queue":"llm.openai"}"#.to_string()),
            json: true,
        }),
        "control heartbeat json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "heartbeat output should be valid json",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let records = must_ok(
        ledger.load_events(&run_id),
        "heartbeat append should persist an event",
    );

    assert_eq!(json["sequence"], 2);
    assert_eq!(json["event"]["run_id"], "run-control-cli");
    assert_eq!(json["event"]["kind"]["event"], "worker_heartbeat_observed");
    assert_eq!(
        json["event"]["kind"]["heartbeat"]["worker_id"],
        "worker-control"
    );
    assert_eq!(
        json["event"]["kind"]["heartbeat"]["metadata"]["queue"],
        "llm.openai"
    );
    assert_eq!(records.len(), 2);
    Ok(())
}

#[test]
fn run_control_heartbeat_renders_text() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Heartbeat {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            worker_id: "worker-control".to_string(),
            observed_at_ms: 20_000,
            expires_at_ms: 35_000,
            metadata: None,
            json: false,
        }),
        "control heartbeat text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control Heartbeat"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Worker: `worker-control`"));
    assert!(output.rendered.contains("- Observed at ms: `20000`"));
    assert!(output.rendered.contains("- Expires at ms: `35000`"));
    Ok(())
}

#[test]
fn run_control_heartbeat_rejects_invalid_metadata_without_append() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::Heartbeat {
        ledger_path: ledger_path.clone(),
        run_id: run_id.as_str().to_string(),
        worker_id: "worker-control".to_string(),
        observed_at_ms: 20_000,
        expires_at_ms: 35_000,
        metadata: Some("{not-json".to_string()),
        json: false,
    }) else {
        return Err("invalid heartbeat metadata should fail".to_string());
    };
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let records = must_ok(
        ledger.load_events(&run_id),
        "invalid heartbeat metadata should not append an event",
    );

    assert!(
        error
            .to_string()
            .contains("invalid `--metadata` JSON for `control heartbeat`")
    );
    assert_eq!(records.len(), 1);
    Ok(())
}

#[test]
fn run_control_query_state_renders_json_without_appending() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_timer(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let before_count = must_ok(
        ledger.load_events(&run_id),
        "should load events before state query",
    )
    .len();

    let output = must_ok(
        run_control_command(&ControlCliCommand::QueryState {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            now_ms: 9_000,
            json: true,
        }),
        "control query state json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "state query output should be valid json",
    );
    let after_count = must_ok(
        ledger.load_events(&run_id),
        "should load events after state query",
    )
    .len();

    assert_eq!(json["event_count"], before_count);
    assert_eq!(json["run_view"]["run_id"], "run-control-cli");
    assert_eq!(
        json["run_view"]["steps"]["run-control-step"]["timers"]["timer-step-approval-timeout"]["status"],
        "scheduled"
    );
    assert_eq!(json["recovery_snapshot"]["observed_at_ms"], 9_000);
    assert_eq!(before_count, after_count);
    Ok(())
}

#[test]
fn run_control_query_state_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step_signal_and_timer(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::QueryState {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 30_000,
            json: false,
        }),
        "control query state text should render",
    );

    assert!(output.rendered.starts_with("# Qianji Control State"));
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Status: `draft`"));
    assert!(output.rendered.contains("- Steps: `1`"));
    assert!(output.rendered.contains("- Step timers: `1`"));
    assert!(output.rendered.contains("- Step signals: `1`"));
    assert!(output.rendered.contains("- Fireable timers: `1`"));
    Ok(())
}

#[test]
fn run_control_recovery_snapshot_renders_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::RecoverySnapshot {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 9_000,
            json: true,
        }),
        "control recovery snapshot json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "snapshot output should be valid json",
    );

    assert_eq!(json["run_id"], "run-control-cli");
    assert_eq!(json["observed_at_ms"], 9_000);
    assert_eq!(json["summary"]["total_actions"], 0);
    assert_eq!(json["plan"]["actions"].as_array().map(Vec::len), Some(0));
    Ok(())
}

#[test]
fn run_control_recovery_snapshot_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::RecoverySnapshot {
            ledger_path,
            run_id: run_id.as_str().to_string(),
            now_ms: 9_000,
            json: false,
        }),
        "control recovery snapshot text should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Control Recovery Snapshot")
    );
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Total actions: `0`"));
    Ok(())
}
