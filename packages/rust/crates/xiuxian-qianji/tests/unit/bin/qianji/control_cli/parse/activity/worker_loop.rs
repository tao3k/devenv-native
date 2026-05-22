use std::path::PathBuf;

use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ControlCliCommand, parse_control_command,
};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_worker_loop_complete_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-worker-loop",
                    "--ledger",
                    "control.duckdb",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--worker-id",
                    "worker-loop",
                    "--task-queue",
                    "llm.openai",
                    "--now-ms",
                    "12345",
                    "--now-step-ms",
                    "10",
                    "--lease-ttl-ms",
                    "500",
                    "--heartbeat-ttl-ms",
                    "250",
                    "--poll-limit",
                    "3",
                    "--empty-limit",
                    "1",
                    "--executor",
                    "fixture",
                    "--outcome",
                    "complete",
                    "--settled-at-ms",
                    "23456",
                    "--settled-step-ms",
                    "20",
                    "--output-hash",
                    "sha256:activity-output",
                    "--metadata",
                    "{\"rows\":3}",
                    "--json",
                ])),
                "control activity-worker-loop complete parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityWorkerLoop {
            ledger_path: PathBuf::from("control.duckdb"),
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            worker_id: "worker-loop".to_string(),
            task_queue: Some("llm.openai".to_string()),
            now_ms: 12_345,
            now_step_ms: 10,
            lease_ttl_ms: 500,
            heartbeat_ttl_ms: Some(250),
            poll_limit: 3,
            empty_limit: 1,
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            settled_at_ms: 23_456,
            settled_step_ms: 20,
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
fn parse_control_activity_worker_loop_rejects_zero_poll_limit() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-worker-loop",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-loop",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "500",
        "--poll-limit",
        "0",
        "--executor",
        "fixture",
        "--outcome",
        "complete",
        "--settled-at-ms",
        "23456",
    ]));
    let error = match result {
        Ok(value) => panic!("zero poll limit should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("invalid `--poll-limit` for `control activity-worker-loop`"),
        "unexpected error: {error}"
    );
}
