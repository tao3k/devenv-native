use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_decision_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "decision",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--decision-id",
                    "decision-doc",
                    "--json",
                ])),
                "control decision parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Decision {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            decision_id: "decision-doc".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_decision_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "decision",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--decision-id",
                    "decision-run",
                ])),
                "control decision parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Decision {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            decision_id: "decision-run".to_string(),
            json: false,
        },
    );
}
