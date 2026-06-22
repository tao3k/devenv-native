use std::path::PathBuf;

use crate::qianji_cli::test_exports::{
    ActivitySettleOutcomeArg, ControlCliCommand, parse_control_command,
};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_settle_complete_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-settle",
                    "--ledger",
                    "control.duckdb",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--leased-task-json",
                    "{\"lease\":{\"lease_id\":\"lease-1\"}}",
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
                "control activity-settle parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivitySettle {
            ledger_path: PathBuf::from("control.duckdb"),
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            leased_task_json: "{\"lease\":{\"lease_id\":\"lease-1\"}}".to_string(),
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
fn parse_control_activity_settle_fail_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-settle",
                    "--ledger",
                    "control.duckdb",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--leased-task-json",
                    "{\"lease\":{\"lease_id\":\"lease-1\"}}",
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
                "control activity-settle fail parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivitySettle {
            ledger_path: PathBuf::from("control.duckdb"),
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: None,
            leased_task_json: "{\"lease\":{\"lease_id\":\"lease-1\"}}".to_string(),
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
fn parse_control_activity_settle_rejects_fail_without_retryable() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-settle",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--leased-task-json",
        "{\"lease\":{\"lease_id\":\"lease-1\"}}",
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
        Ok(value) => panic!("missing activity-settle retryable should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error.to_string().contains(
            "missing `--retryable <true|false>` for `control activity-settle --outcome fail`"
        ),
        "unexpected error: {error}"
    );
}
