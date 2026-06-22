use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_llm_activities_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "llm-activities",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--require-request-audit",
                    "--json",
                ])),
                "control llm-activities parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::LlmActivities {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            require_request_audit: true,
            json: true,
        },
    );
}
