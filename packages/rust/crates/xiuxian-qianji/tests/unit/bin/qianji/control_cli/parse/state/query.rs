use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_query_state_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "query",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--state",
                    "--now-ms",
                    "1234",
                    "--json",
                ])),
                "control query state parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::QueryState {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            now_ms: 1234,
            json: true,
        },
    );
}

#[test]
fn parse_control_query_rejects_missing_state_flag() {
    let Err(error) = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "query",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control",
        "--now-ms",
        "1234",
    ])) else {
        panic!("missing query kind should fail");
    };

    assert!(error.to_string().contains("missing `--state`"));
}
