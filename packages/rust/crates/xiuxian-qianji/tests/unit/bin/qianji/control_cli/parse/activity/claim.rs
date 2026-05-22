use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_claim_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-claim",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--worker-id",
                    "worker-claim",
                    "--task-queue",
                    "llm.openai",
                    "--now-ms",
                    "12345",
                    "--lease-ttl-ms",
                    "30000",
                    "--json",
                ])),
                "control activity-claim parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityClaim {
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            worker_id: "worker-claim".to_string(),
            task_queue: Some("llm.openai".to_string()),
            now_ms: 12_345,
            lease_ttl_ms: 30_000,
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_claim_rejects_missing_worker_id() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-claim",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "30000",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity claim worker should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--worker-id <id>` for `control activity-claim`"),
        "unexpected error: {error}"
    );
}
