use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_reclaim_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-reclaim",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--lease-json",
                    r#"{"lease_id":"lease-a","run_id":"run-a","activity_id":"activity-a","worker_id":"worker-a","acquired_at_ms":10,"expires_at_ms":20}"#,
                    "--now-ms",
                    "21",
                    "--json",
                ])),
                "control activity-reclaim parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityReclaim {
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            lease_json: r#"{"lease_id":"lease-a","run_id":"run-a","activity_id":"activity-a","worker_id":"worker-a","acquired_at_ms":10,"expires_at_ms":20}"#.to_string(),
            now_ms: 21,
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_reclaim_rejects_missing_now_ms() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-reclaim",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--lease-json",
        r#"{"lease_id":"lease-a"}"#,
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity reclaim now-ms should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--now-ms <ms>` for `control activity-reclaim`"),
        "unexpected error: {error}"
    );
}
