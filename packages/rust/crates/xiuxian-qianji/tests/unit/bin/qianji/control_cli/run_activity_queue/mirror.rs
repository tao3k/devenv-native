#[cfg(all(feature = "duckdb", feature = "valkey"))]
use crate::qianji_cli::test_exports::{ControlCliCommand, handle_control_command_async};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::test_exports::{WorkerActivityMirrorStoreRequest, mirror_with_hot_state};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::tests::control_cli::support::must_some;
use crate::qianji_cli::tests::control_cli::support::{
    append_control_run_with_scheduled_activity_queue, must_ok,
};
use tempfile::TempDir;
use xiuxian_qianji_control::{
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore, TaskQueue, WorkerId,
    WorkerRef,
};

#[tokio::test]
async fn mirror_with_hot_state_enqueues_replay_tasks_without_appending() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    let before_count = must_ok(
        ledger.load_events(&run_id),
        "should load events before activity mirror",
    )
    .len();

    let output = must_ok(
        mirror_with_hot_state(
            &ledger,
            &hot_state,
            WorkerActivityMirrorStoreRequest {
                run_id: run_id.as_str(),
                task_queue: Some("llm.openai"),
                priority: 7,
                not_before_ms: 123,
                metadata: Some(r#"{"source":"activity-mirror-test"}"#),
                json: true,
            },
        )
        .await,
        "activity mirror should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity mirror output should be valid json",
    );
    let after_count = must_ok(
        ledger.load_events(&run_id),
        "should load events after activity mirror",
    )
    .len();

    assert_eq!(json["run_id"], run_id.as_str());
    assert_eq!(json["task_queue"], "llm.openai");
    assert_eq!(json["mirrored_count"], 1);
    assert_eq!(before_count, after_count);

    let task_queue = must_ok(TaskQueue::new("llm.openai"), "should build task queue");
    let claimed_too_early = must_ok(
        hot_state
            .claim_activity_task(
                WorkerRef {
                    worker_id: must_ok(WorkerId::new("worker-early"), "worker id"),
                    capabilities: Vec::new(),
                    metadata: serde_json::Value::Null,
                },
                Some(&task_queue),
                122,
                50,
            )
            .await,
        "early claim should not fail",
    );
    assert_eq!(claimed_too_early, None);

    let claimed = must_ok(
        hot_state
            .claim_activity_task(
                WorkerRef {
                    worker_id: must_ok(WorkerId::new("worker-mirror"), "worker id"),
                    capabilities: Vec::new(),
                    metadata: serde_json::Value::Null,
                },
                Some(&task_queue),
                123,
                50,
            )
            .await,
        "claim should read mirrored task",
    );
    let claimed = claimed.ok_or_else(|| "expected mirrored task to be claimable".to_string())?;

    assert_eq!(
        claimed.activity_task.task.activity_id.as_str(),
        "activity-run-scheduled"
    );
    assert_eq!(claimed.activity_task.priority, 7);
    assert_eq!(claimed.activity_task.not_before_ms, 123);
    assert_eq!(
        claimed.activity_task.metadata,
        serde_json::json!({"source": "activity-mirror-test"})
    );
    Ok(())
}

#[tokio::test]
async fn mirror_with_hot_state_renders_text_summary() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();

    let output = must_ok(
        mirror_with_hot_state(
            &ledger,
            &hot_state,
            WorkerActivityMirrorStoreRequest {
                run_id: run_id.as_str(),
                task_queue: Some("tool.github"),
                priority: 0,
                not_before_ms: 0,
                metadata: None,
                json: false,
            },
        )
        .await,
        "activity mirror text should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Control Activity Mirror")
    );
    assert!(output.rendered.contains("- Run: `run-control-cli`"));
    assert!(output.rendered.contains("- Task queue: `tool.github`"));
    assert!(output.rendered.contains("- Mirrored tasks: `1`"));
    Ok(())
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
#[tokio::test]
async fn activity_mirror_async_handler_does_not_start_runtime_inside_runtime() -> Result<(), String>
{
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_scheduled_activity_queue(&ledger_path);

    let result = handle_control_command_async(ControlCliCommand::ActivityMirror {
        ledger_path,
        valkey_url: "redis://127.0.0.1:1/0".to_string(),
        namespace: Some("mirror-runtime-regression".to_string()),
        run_id: run_id.to_string(),
        task_queue: Some("llm.openai".to_string()),
        priority: 0,
        not_before_ms: 0,
        metadata: None,
        json: true,
    })
    .await;
    let error = match result {
        Ok(()) => return Err("unreachable valkey should return a normal connection error".into()),
        Err(error) => error,
    };

    assert!(
        !error
            .to_string()
            .contains("Cannot start a runtime from within a runtime"),
        "control dispatch should isolate sync runtime users from the CLI runtime: {error}"
    );
    Ok(())
}

#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
#[test]
fn run_control_activity_mirror_requires_duckdb_and_valkey_without_connecting() {
    let error = must_some(
        run_control_command(&ControlCliCommand::ActivityMirror {
            ledger_path: "control.duckdb".into(),
            valkey_url: "redis://127.0.0.1:1".to_string(),
            namespace: None,
            run_id: "run-control".to_string(),
            task_queue: Some("llm.openai".to_string()),
            priority: 0,
            not_before_ms: 0,
            metadata: None,
            json: true,
        })
        .err(),
        "activity mirror should require duckdb and valkey features",
    );

    assert!(
        error
            .to_string()
            .contains("`control activity-mirror` requires the `duckdb` and `valkey` features"),
        "unexpected error: {error}"
    );
}
