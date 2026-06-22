use std::path::PathBuf;

use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_take_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-take",
                    "--ledger",
                    "control.duckdb",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--worker-id",
                    "worker-take",
                    "--task-queue",
                    "llm.openai",
                    "--now-ms",
                    "12345",
                    "--lease-ttl-ms",
                    "30000",
                    "--json",
                ])),
                "control activity-take parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityTake {
            ledger_path: PathBuf::from("control.duckdb"),
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            worker_id: "worker-take".to_string(),
            task_queue: Some("llm.openai".to_string()),
            now_ms: 12_345,
            lease_ttl_ms: 30_000,
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_take_rejects_missing_ledger() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-take",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-take",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "30000",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity take ledger should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--ledger <path>` for `control activity-take`"),
        "unexpected error: {error}"
    );
}
