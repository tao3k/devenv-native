#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ControlCliCommand, run_control_command,
};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::tests::control_cli::support::{
    append_empty_control_run, must_ok, must_some,
};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use tempfile::TempDir;

#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
#[test]
fn run_control_activity_worker_once_requires_duckdb_and_valkey_features_without_connecting() {
    let temp_dir = must_ok(TempDir::new(), "should create temporary directory");
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);
    let error = must_some(
        run_control_command(&ControlCliCommand::ActivityWorkerOnce {
            ledger_path,
            valkey_url: "redis://127.0.0.1:1".to_string(),
            namespace: None,
            worker_id: "worker-once".to_string(),
            task_queue: Some("llm.openai".to_string()),
            now_ms: 10,
            lease_ttl_ms: 50,
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            settled_at_ms: 20,
            output_hash: Some("sha256:activity-output".to_string()),
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
            json: true,
        })
        .err(),
        "activity worker once should require duckdb and valkey features in partial builds",
    );

    assert!(
        error
            .to_string()
            .contains("`control activity-worker-once` requires the `duckdb` and `valkey` features"),
        "unexpected error for run {run_id}: {error}"
    );
}
