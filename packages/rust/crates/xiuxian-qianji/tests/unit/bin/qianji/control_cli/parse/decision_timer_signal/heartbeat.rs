use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

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
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
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
            valkey_url: Some("redis://127.0.0.1:6379".to_string()),
            namespace: Some("qianji:test".to_string()),
            run_id: "run-control".to_string(),
            worker_id: "worker-a".to_string(),
            observed_at_ms: 1_000,
            expires_at_ms: 3_000,
            metadata: Some(r#"{"queue":"llm.openai"}"#.to_string()),
            json: true,
        },
    );
}
