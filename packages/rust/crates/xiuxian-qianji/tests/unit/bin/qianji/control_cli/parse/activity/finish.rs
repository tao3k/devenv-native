use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_complete_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-complete",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--activity-id",
                    "activity-step-llm",
                    "--completed-at-ms",
                    "23456",
                    "--output-hash",
                    "sha256:activity-output",
                    "--metadata",
                    "{\"rows\":3}",
                    "--json",
                ])),
                "control activity-complete parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityComplete {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            activity_id: "activity-step-llm".to_string(),
            completed_at_ms: 23_456,
            output_hash: Some("sha256:activity-output".to_string()),
            metadata: Some("{\"rows\":3}".to_string()),
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_fail_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-fail",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--activity-id",
                    "activity-run-llm",
                    "--failed-at-ms",
                    "34567",
                    "--error-code",
                    "rate_limited",
                    "--message",
                    "provider rejected request",
                    "--retryable",
                    "true",
                    "--attempt",
                    "2",
                    "--json",
                ])),
                "control activity-fail parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityFail {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            activity_id: "activity-run-llm".to_string(),
            failed_at_ms: 34_567,
            error_code: "rate_limited".to_string(),
            message: "provider rejected request".to_string(),
            retryable: true,
            attempt: 2,
            metadata: None,
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_fail_rejects_missing_retryable() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-fail",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control",
        "--activity-id",
        "activity-run-llm",
        "--failed-at-ms",
        "34567",
        "--error-code",
        "rate_limited",
        "--message",
        "provider rejected request",
        "--attempt",
        "2",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity-fail retryable should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--retryable <true|false>` for `control activity-fail`"),
        "unexpected error: {error}"
    );
}
