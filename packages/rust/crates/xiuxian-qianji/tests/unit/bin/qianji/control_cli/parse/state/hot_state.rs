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
