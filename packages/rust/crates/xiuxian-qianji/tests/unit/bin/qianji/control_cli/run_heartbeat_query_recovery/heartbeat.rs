use crate::qianji_cli::test_exports::{
    ControlCliCommand, HeartbeatHotStateRequest, heartbeat_with_hot_state, run_control_command,
};
use crate::qianji_cli::tests::control_cli::support::{append_empty_control_run, must_ok};
use tempfile::TempDir;
use xiuxian_qianji_control::{
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore,
};

#[test]
fn run_control_heartbeat_appends_json() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);

    let output = must_ok(
        run_control_command(&ControlCliCommand::Heartbeat {
            ledger_path: ledger_path.clone(),
            valkey_url: None,
            namespace: None,
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

#[tokio::test]
async fn heartbeat_with_hot_state_records_liveness_before_durable_event() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();

    let output = must_ok(
        heartbeat_with_hot_state(
            &ledger,
            &hot_state,
            HeartbeatHotStateRequest {
                run_id: run_id.as_str(),
                worker_id: "worker-control",
                observed_at_ms: 20_000,
                expires_at_ms: 35_000,
                metadata: Some(r#"{"queue":"llm.openai"}"#),
                json: true,
            },
        )
        .await,
        "heartbeat hot-state path should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "heartbeat output should be valid json",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(21_000).await,
        "hot-state heartbeat snapshot should load",
    );
    let records = must_ok(
        ledger.load_events(&run_id),
        "heartbeat append should persist an event",
    );

    assert_eq!(json["event"]["kind"]["event"], "worker_heartbeat_observed");
    assert_eq!(snapshot.live_heartbeat_count(), 1);
    assert_eq!(
        snapshot.worker_heartbeats[0].worker_id.as_str(),
        "worker-control"
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
            valkey_url: None,
            namespace: None,
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
        valkey_url: None,
        namespace: None,
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
