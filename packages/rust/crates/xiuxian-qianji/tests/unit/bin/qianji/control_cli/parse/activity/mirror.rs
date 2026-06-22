use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_mirror_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-mirror",
                    "--ledger",
                    "control.duckdb",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--run-id",
                    "run-control",
                    "--task-queue",
                    "llm.openai",
                    "--priority",
                    "7",
                    "--not-before-ms",
                    "12345",
                    "--metadata",
                    r#"{"source":"unit"}"#,
                    "--json",
                ])),
                "control activity-mirror parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityMirror {
            ledger_path: "control.duckdb".into(),
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            run_id: "run-control".to_string(),
            task_queue: Some("llm.openai".to_string()),
            priority: 7,
            not_before_ms: 12_345,
            metadata: Some(r#"{"source":"unit"}"#.to_string()),
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_mirror_rejects_missing_valkey_url() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-mirror",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity mirror valkey url should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--valkey-url <url>` for `control activity-mirror`"),
        "unexpected error: {error}"
    );
}
