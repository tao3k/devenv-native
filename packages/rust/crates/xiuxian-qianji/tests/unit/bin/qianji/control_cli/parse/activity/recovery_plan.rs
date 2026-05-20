use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_apply_recovery_plan_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "apply-recovery-plan",
                    "--ledger",
                    "control.duckdb",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--run-id",
                    "run-control",
                    "--now-ms",
                    "12345",
                    "--attempt",
                    "2",
                    "--reason",
                    "operator recovery",
                    "--max-attempts",
                    "5",
                    "--backoff-ms",
                    "250",
                    "--require-human-approval",
                    "--priority",
                    "9",
                    "--json",
                ])),
                "control apply-recovery-plan parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ApplyRecoveryPlan {
            ledger_path: "control.duckdb".into(),
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            run_id: "run-control".to_string(),
            now_ms: 12_345,
            attempt: 2,
            reason: "operator recovery".to_string(),
            max_attempts: 5,
            backoff_ms: 250,
            require_human_approval: true,
            priority: 9,
            json: true,
        },
    );
}

#[test]
fn parse_control_apply_recovery_plan_rejects_missing_valkey_url() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "apply-recovery-plan",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control",
        "--now-ms",
        "12345",
        "--attempt",
        "1",
        "--reason",
        "operator recovery",
        "--max-attempts",
        "3",
    ]));
    let error = match result {
        Ok(value) => panic!("missing recovery Valkey URL should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--valkey-url <url>` for `control apply-recovery-plan`"),
        "unexpected error: {error}"
    );
}
