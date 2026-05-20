use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--activity-id",
                    "activity-doc",
                    "--json",
                ])),
                "control activity parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Activity {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            activity_id: "activity-doc".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--activity-id",
                    "activity-run",
                ])),
                "control activity parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Activity {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            activity_id: "activity-run".to_string(),
            json: false,
        },
    );
}
