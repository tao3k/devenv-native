use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, to_args};

#[test]
fn parse_control_activity_schedule_llm_command() {
    let command = must_ok(
        parse_control_command(&to_args(&[
            "qianji",
            "control",
            "activity-schedule-llm",
            "--ledger",
            "control.duckdb",
            "--run-id",
            "run-control-cli",
            "--step-id",
            "run-control-step",
            "--occurred-at-ms",
            "42",
            "--llm-activity-json",
            r#"{"task":{},"request":{}}"#,
            "--json",
        ])),
        "control activity-schedule-llm parse should succeed",
    );

    assert_eq!(
        command,
        Some(ControlCliCommand::ActivityScheduleLlm {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control-cli".to_string(),
            step_id: Some("run-control-step".to_string()),
            occurred_at_ms: 42,
            llm_activity_json: r#"{"task":{},"request":{}}"#.to_string(),
            json: true,
        })
    );
}

#[test]
fn parse_control_activity_schedule_llm_requires_payload() {
    let error = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-schedule-llm",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control-cli",
        "--occurred-at-ms",
        "42",
    ]))
    .err()
    .unwrap_or_else(|| panic!("missing llm activity payload should fail"));

    assert!(
        error
            .to_string()
            .contains("missing `--llm-activity-json <json>`"),
        "unexpected error: {error}"
    );
}
