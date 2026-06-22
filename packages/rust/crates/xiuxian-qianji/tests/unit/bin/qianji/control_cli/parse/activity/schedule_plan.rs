use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, to_args};

#[test]
fn parse_control_activity_admit_plan_command() {
    let command = must_ok(
        parse_control_command(&to_args(&[
            "qianji",
            "control",
            "activity-admit-plan",
            "--ledger",
            "control.duckdb",
            "--run-id",
            "run-control-cli",
            "--step-id",
            "run-control-step",
            "--occurred-at-ms",
            "42",
            "--schedule-plan-json",
            "qianji_schedule_plan.json",
            "--json",
        ])),
        "control activity-admit-plan parse should succeed",
    );

    assert_eq!(
        command,
        Some(ControlCliCommand::ActivityAdmitPlan {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control-cli".to_string(),
            step_id: Some("run-control-step".to_string()),
            occurred_at_ms: 42,
            schedule_plan_json_path: "qianji_schedule_plan.json".into(),
            json: true,
        })
    );
}

#[test]
fn parse_control_activity_admit_plan_requires_schedule_plan_path() {
    let error = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-admit-plan",
        "--ledger",
        "control.duckdb",
        "--run-id",
        "run-control-cli",
        "--occurred-at-ms",
        "42",
    ]))
    .err()
    .unwrap_or_else(|| panic!("missing schedule-plan path should fail"));

    assert!(
        error
            .to_string()
            .contains("missing `--schedule-plan-json <path>`"),
        "unexpected error: {error}"
    );
}
