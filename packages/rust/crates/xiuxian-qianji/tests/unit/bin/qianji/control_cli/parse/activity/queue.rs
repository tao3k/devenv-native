use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_queue_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-queue",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--task-queue",
                    "llm.openai",
                    "--json",
                ])),
                "control activity-queue parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityQueue {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            task_queue: Some("llm.openai".to_string()),
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_queue_rejects_missing_ledger() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-queue",
        "--run-id",
        "run-control",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity queue ledger should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--ledger <path>` for `control activity-queue`"),
        "unexpected error: {error}"
    );
}
