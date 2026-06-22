use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, to_args};

#[test]
fn parse_control_run_create_command() {
    let command = must_ok(
        parse_control_command(&to_args(&[
            "qianji",
            "control",
            "run-create",
            "--ledger",
            "control.duckdb",
            "--run-id",
            "run-control-cli",
            "--occurred-at-ms",
            "42",
            "--intent",
            "admit schedule plan",
            "--json",
        ])),
        "control run-create parse should succeed",
    );

    assert_eq!(
        command,
        Some(ControlCliCommand::RunCreate {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control-cli".to_string(),
            occurred_at_ms: 42,
            intent: "admit schedule plan".to_string(),
            json: true,
        })
    );
}

#[test]
fn parse_control_run_create_requires_intent() {
    let error = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "run-create",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control-cli",
        "--occurred-at-ms",
        "42",
    ]))
    .err()
    .unwrap_or_else(|| panic!("missing run-create intent should fail"));

    assert!(
        error.to_string().contains("missing `--intent <text>`"),
        "unexpected error: {error}"
    );
}
