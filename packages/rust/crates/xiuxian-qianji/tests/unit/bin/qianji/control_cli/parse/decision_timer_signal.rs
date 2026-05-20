use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_decision_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "decision",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--decision-id",
                    "decision-doc",
                    "--json",
                ])),
                "control decision parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Decision {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            decision_id: "decision-doc".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_decision_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "decision",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--decision-id",
                    "decision-run",
                ])),
                "control decision parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Decision {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            decision_id: "decision-run".to_string(),
            json: false,
        },
    );
}

#[test]
fn parse_control_timer_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "timer",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--timer-id",
                    "timer-doc",
                    "--json",
                ])),
                "control timer parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Timer {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            timer_id: "timer-doc".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_timer_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "timer",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--timer-id",
                    "timer-run",
                ])),
                "control timer parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Timer {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            timer_id: "timer-run".to_string(),
            json: false,
        },
    );
}

#[test]
fn parse_control_signal_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "signal",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--signal-name",
                    "human.approval",
                    "--payload",
                    r#"{"approved":true}"#,
                    "--received-at-ms",
                    "12345",
                    "--json",
                ])),
                "control signal parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Signal {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: Some("run-control-step".to_string()),
            signal_name: "human.approval".to_string(),
            payload: r#"{"approved":true}"#.to_string(),
            received_at_ms: 12_345,
            json: true,
        },
    );
}

#[test]
fn parse_control_signal_command_without_step_scope() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "signal",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--signal-name",
                    "run.refresh",
                    "--payload",
                    r#"{"reason":"manual"}"#,
                    "--received-at-ms",
                    "54321",
                ])),
                "control signal parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Signal {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: None,
            signal_name: "run.refresh".to_string(),
            payload: r#"{"reason":"manual"}"#.to_string(),
            received_at_ms: 54_321,
            json: false,
        },
    );
}

#[test]
fn parse_control_heartbeat_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "heartbeat",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--worker-id",
                    "worker-a",
                    "--observed-at-ms",
                    "1000",
                    "--expires-at-ms",
                    "3000",
                    "--metadata",
                    r#"{"queue":"llm.openai"}"#,
                    "--json",
                ])),
                "control heartbeat parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Heartbeat {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            worker_id: "worker-a".to_string(),
            observed_at_ms: 1_000,
            expires_at_ms: 3_000,
            metadata: Some(r#"{"queue":"llm.openai"}"#.to_string()),
            json: true,
        },
    );
}
