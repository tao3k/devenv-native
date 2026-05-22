use std::path::PathBuf;

use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ControlCliCommand, parse_control_command,
};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_worker_once_complete_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-worker-once",
                    "--ledger",
                    "control.duckdb",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--worker-id",
                    "worker-once",
                    "--task-queue",
                    "llm.openai",
                    "--now-ms",
                    "12345",
                    "--lease-ttl-ms",
                    "500",
                    "--executor",
                    "fixture",
                    "--outcome",
                    "complete",
                    "--settled-at-ms",
                    "23456",
                    "--output-hash",
                    "sha256:activity-output",
                    "--metadata",
                    "{\"rows\":3}",
                    "--json",
                ])),
                "control activity-worker-once complete parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityWorkerOnce {
            ledger_path: PathBuf::from("control.duckdb"),
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            worker_id: "worker-once".to_string(),
            task_queue: Some("llm.openai".to_string()),
            now_ms: 12_345,
            lease_ttl_ms: 500,
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            settled_at_ms: 23_456,
            output_hash: Some("sha256:activity-output".to_string()),
            error_code: None,
            message: None,
            retryable: None,
            metadata: Some("{\"rows\":3}".to_string()),
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_worker_once_fail_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-worker-once",
                    "--ledger",
                    "control.duckdb",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--worker-id",
                    "worker-once",
                    "--now-ms",
                    "12345",
                    "--lease-ttl-ms",
                    "500",
                    "--executor",
                    "fixture",
                    "--outcome",
                    "fail",
                    "--settled-at-ms",
                    "23456",
                    "--error-code",
                    "rate_limited",
                    "--message",
                    "provider rejected request",
                    "--retryable",
                    "true",
                    "--json",
                ])),
                "control activity-worker-once fail parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityWorkerOnce {
            ledger_path: PathBuf::from("control.duckdb"),
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: None,
            worker_id: "worker-once".to_string(),
            task_queue: None,
            now_ms: 12_345,
            lease_ttl_ms: 500,
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Fail,
            settled_at_ms: 23_456,
            output_hash: None,
            error_code: Some("rate_limited".to_string()),
            message: Some("provider rejected request".to_string()),
            retryable: Some(true),
            metadata: None,
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_worker_once_rejects_fail_without_retryable() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-worker-once",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-once",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "500",
        "--executor",
        "fixture",
        "--outcome",
        "fail",
        "--settled-at-ms",
        "23456",
        "--error-code",
        "rate_limited",
        "--message",
        "provider rejected request",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity-worker-once retryable should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error.to_string().contains(
            "missing `--retryable <true|false>` for `control activity-worker-once --outcome fail`"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_control_activity_worker_once_rejects_unknown_executor() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-worker-once",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-once",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "500",
        "--executor",
        "real",
        "--outcome",
        "complete",
        "--settled-at-ms",
        "23456",
    ]));
    let error = match result {
        Ok(value) => panic!("unknown activity-worker-once executor should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("invalid `--executor` for `control activity-worker-once`"),
        "unexpected error: {error}"
    );
}
