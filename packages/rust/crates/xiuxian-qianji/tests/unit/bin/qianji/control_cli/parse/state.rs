use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_hot_state_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "hot-state",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--now-ms",
                    "12345",
                    "--json",
                ])),
                "control hot-state parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::HotState {
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            now_ms: 12_345,
            json: true,
        },
    );
}

#[test]
fn parse_control_hot_state_rejects_missing_now_ms() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "hot-state",
        "--valkey-url",
        "redis://127.0.0.1:6379",
    ]));
    let error = match result {
        Ok(value) => panic!("missing hot-state timestamp should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--now-ms <ms>` for `control hot-state`"),
        "unexpected error: {error}"
    );
}

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

#[test]
fn parse_control_history_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "history",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--json",
                ])),
                "control history parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::History {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_recovery_snapshot_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "recovery-snapshot",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--now-ms",
                    "1234",
                    "--json",
                ])),
                "control recovery snapshot parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::RecoverySnapshot {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            now_ms: 1234,
            json: true,
        },
    );
}

#[test]
fn parse_control_view_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "view",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--json",
                ])),
                "control view parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::View {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_step_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "step",
                    "--ledger",
                    "control.duckdb",
                    "--run-id",
                    "run-control",
                    "--step-id",
                    "run-control-step",
                    "--json",
                ])),
                "control step parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::Step {
            ledger_path: "control.duckdb".into(),
            run_id: "run-control".to_string(),
            step_id: "run-control-step".to_string(),
            json: true,
        },
    );
}

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
