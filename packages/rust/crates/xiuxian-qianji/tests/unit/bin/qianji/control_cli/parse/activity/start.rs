use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_start_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-start",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--activity-id",
                    "activity-step-llm",
                    "--worker-id",
                    "worker-llm",
                    "--started-at-ms",
                    "12345",
                    "--attempt",
                    "2",
                    "--json",
                ])),
                "control activity-start parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityStart {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            activity_id: "activity-step-llm".to_string(),
            worker_id: "worker-llm".to_string(),
            started_at_ms: 12_345,
            attempt: 2,
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_start_rejects_missing_worker() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-start",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control",
        "--activity-id",
        "activity-step-llm",
        "--started-at-ms",
        "12345",
        "--attempt",
        "1",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity-start worker should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--worker-id <id>` for `control activity-start`"),
        "unexpected error: {error}"
    );
}
