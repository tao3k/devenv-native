use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_summary_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "summary",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--now-ms",
                    "25000",
                    "--json",
                ])),
                "control summary parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Summary {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            now_ms: 25_000,
            json: true,
        },
    );
}

#[test]
fn parse_control_summary_rejects_missing_now_ms() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "summary",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control",
    ]));

    assert!(
        result
            .expect_err("summary without now-ms should fail")
            .to_string()
            .contains("missing `--now-ms <ms>` for `control summary`")
    );
}
