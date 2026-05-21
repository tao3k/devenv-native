use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_signal_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "signal",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--signal-name",
                    "human.approval",
                    "--payload",
                    r#"{"approved":true}"#,
                    "--received-at-ms",
                    "12345",
                    "--json",
                ])),
                "control signal parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Signal {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            signal_name: "human.approval".to_string(),
            payload: r#"{"approved":true}"#.to_string(),
            received_at_ms: 12_345,
            json: true,
        },
    );
}

#[test]
fn parse_control_signal_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "signal",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--signal-name",
                    "run.refresh",
                    "--payload",
                    r#"{"reason":"manual"}"#,
                    "--received-at-ms",
                    "54321",
                ])),
                "control signal parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Signal {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            signal_name: "run.refresh".to_string(),
            payload: r#"{"reason":"manual"}"#.to_string(),
            received_at_ms: 54_321,
            json: false,
        },
    );
}

#[test]
fn parse_control_signals_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "signals",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--json",
                ])),
                "control signals parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Signals {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            json: true,
        },
    );
}
