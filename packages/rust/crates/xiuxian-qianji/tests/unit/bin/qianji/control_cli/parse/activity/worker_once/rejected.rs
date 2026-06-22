use crate::qianji_cli::test_exports::parse_control_command;
use crate::qianji_cli::tests::control_cli::support::to_args;

#[test]
fn parse_control_activity_worker_once_rejects_fail_without_retryable() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-worker-once",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-once",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "500",
        "--executor",
        "fixture",
        "--outcome",
        "fail",
        "--settled-at-ms",
        "23456",
        "--error-code",
        "rate_limited",
        "--message",
        "provider rejected request",
    ]));
    let error = match result {
        Ok(value) => panic!("missing activity-worker-once retryable should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error.to_string().contains(
            "missing `--retryable <true|false>` for `control activity-worker-once --outcome fail`"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_control_activity_worker_once_rejects_openai_without_output_artifact() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-worker-once",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-once",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "500",
        "--executor",
        "openai-compatible-llm",
        "--outcome",
        "complete",
        "--settled-at-ms",
        "23456",
        "--openai-compatible-base-url",
        "http://127.0.0.1:8080/v1",
    ]));
    let error = match result {
        Ok(value) => {
            panic!("OpenAI-compatible executor without artifact should fail, got {value:?}")
        }
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("missing `--output-artifact-path <path>`"),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_control_activity_worker_once_rejects_fail_with_output_ref() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-worker-once",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-once",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "500",
        "--executor",
        "fixture",
        "--outcome",
        "fail",
        "--settled-at-ms",
        "23456",
        "--error-code",
        "rate_limited",
        "--message",
        "provider rejected request",
        "--retryable",
        "true",
        "--output-ref-json",
        r#"{"artifact_id":"artifact-worker-output","artifact_kind":"llm.output","uri":"artifact://artifact-worker-output"}"#,
    ]));
    let error = match result {
        Ok(value) => panic!("fail with output ref should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("cannot be combined with output artifact or output reference arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_control_activity_worker_once_rejects_fail_with_output_artifact() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-worker-once",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-once",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "500",
        "--executor",
        "fixture",
        "--outcome",
        "fail",
        "--settled-at-ms",
        "23456",
        "--error-code",
        "rate_limited",
        "--message",
        "provider rejected request",
        "--retryable",
        "true",
        "--output-artifact-path",
        "artifacts/activity-output.json",
        "--output-artifact-content",
        "{\"answer\":\"done\"}",
    ]));
    let error = match result {
        Ok(value) => panic!("fail with output artifact should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("cannot be combined with output artifact or output reference arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_control_activity_worker_once_rejects_output_artifact_without_content() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-worker-once",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-once",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "500",
        "--executor",
        "fixture",
        "--outcome",
        "complete",
        "--settled-at-ms",
        "23456",
        "--output-artifact-path",
        "artifacts/activity-output.json",
    ]));
    let error = match result {
        Ok(value) => panic!("output artifact without content should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error.to_string().contains(
            "missing `--output-artifact-content <text>` for `control activity-worker-once`"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_control_activity_worker_once_rejects_unknown_executor() {
    let result = parse_control_command(&to_args(&[
        "qianji",
        "control",
        "activity-worker-once",
        "--ledger",
        "control.duckdb",
        "--valkey-url",
        "redis://127.0.0.1:6379",
        "--worker-id",
        "worker-once",
        "--now-ms",
        "12345",
        "--lease-ttl-ms",
        "500",
        "--executor",
        "real",
        "--outcome",
        "complete",
        "--settled-at-ms",
        "23456",
    ]));
    let error = match result {
        Ok(value) => panic!("unknown activity-worker-once executor should fail, got {value:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("invalid `--executor` for `control activity-worker-once`"),
        "unexpected error: {error}"
    );
}
