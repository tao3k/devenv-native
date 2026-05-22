use crate::qianji_cli::test_exports::{ControlCliCommand, parse_control_command};
use crate::qianji_cli::tests::control_cli::support::{must_ok, must_some, to_args};

#[test]
fn parse_control_activity_release_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_control_command(&to_args(&[
                    "qianji",
                    "control",
                    "activity-release",
                    "--valkey-url",
                    "redis://127.0.0.1:6379",
                    "--namespace",
                    "qianji:test",
                    "--lease-json",
                    r#"{"lease_id":"lease-a","run_id":"run-a","activity_id":"activity-a","worker_id":"worker-a","acquired_at_ms":10,"expires_at_ms":20}"#,
                    "--json",
                ])),
                "control activity-release parse should succeed",
            ),
            "control command should be detected",
        ),
        ControlCliCommand::ActivityRelease {
            valkey_url: "redis://127.0.0.1:6379".to_string(),
            namespace: Some("qianji:test".to_string()),
            lease_json: r#"{"lease_id":"lease-a","run_id":"run-a","activity_id":"activity-a","worker_id":"worker-a","acquired_at_ms":10,"expires_at_ms":20}"#.to_string(),
            json: true,
        },
    );
}

#[test]
fn parse_control_activity_release_rejects_missing_lease_json() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-release",
        "--valkey-url",
        "redis://127.0.0.1:6379",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity release lease json should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--lease-json <json>` for `control activity-release`"),
        "unexpected error: {error}"
    );
}
