use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_timer_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "timer",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--timer-id",
                    "timer-doc",
                    "--json",
                ])),
                "control timer parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Timer {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            timer_id: "timer-doc".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_timer_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "timer",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--timer-id",
                    "timer-run",
                ])),
                "control timer parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Timer {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            timer_id: "timer-run".to_string(),
            json: false,
        },
    );
}

#[test]
fn parse_control_timers_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "timers",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--json",
                ])),
                "control timers parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Timers {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            json: true,
        },
    );
}
