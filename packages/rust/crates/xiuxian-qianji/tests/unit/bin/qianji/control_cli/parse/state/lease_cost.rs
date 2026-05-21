use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_lease_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "lease",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--json",
                ])),
                "control lease parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Lease {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: "run-control-step".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_leases_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "leases",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--json",
                ])),
                "control leases parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Leases {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_costs_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "costs",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--json",
                ])),
                "control costs parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Costs {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            json: true,
        },
    );
}
