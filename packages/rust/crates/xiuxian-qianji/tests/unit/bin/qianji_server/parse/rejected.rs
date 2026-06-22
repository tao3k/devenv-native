use crate::qianji_server_cli::cli::parse_qianji_server_args;

use crate::qianji_server_cli::tests::support::must_err;

#[test]
fn qianji_server_rejects_missing_bind_value() {
    let error = must_err(
        parse_qianji_server_args(["--bind"]),
        "missing bind value should be rejected",
    );

    assert!(
        error.contains("missing value for --bind"),
        "unexpected error: {error}"
    );
}

#[test]
fn qianji_server_rejects_missing_flight_bind_value() {
    let error = must_err(
        parse_qianji_server_args(["--flight-bind"]),
        "missing Flight bind value should be rejected",
    );

    assert!(
        error.contains("missing value for --flight-bind"),
        "unexpected error: {error}"
    );
}

#[test]
fn qianji_server_rejects_missing_valkey_url() {
    let error = must_err(
        parse_qianji_server_args(["--valkey-url"]),
        "missing valkey URL should be rejected",
    );

    assert!(
        error.contains("missing value for --valkey-url"),
        "unexpected error: {error}"
    );
}

#[test]
fn qianji_server_rejects_missing_flowhub_root() {
    let error = must_err(
        parse_qianji_server_args(["--flowhub-root"]),
        "missing Flowhub root should be rejected",
    );

    assert!(
        error.contains("missing value for --flowhub-root"),
        "unexpected error: {error}"
    );
}

#[test]
fn qianji_server_rejects_missing_control_ledger_path() {
    let error = must_err(
        parse_qianji_server_args(["--control-ledger"]),
        "missing control ledger path should be rejected",
    );

    assert!(
        error.contains("missing value for --control-ledger"),
        "unexpected error: {error}"
    );
}

#[test]
fn qianji_server_rejects_empty_valkey_url() {
    let error = must_err(
        parse_qianji_server_args(["--valkey-url", "  "]),
        "empty valkey URL should be rejected",
    );

    assert!(
        error.contains("--valkey-url must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn qianji_server_rejects_empty_flowhub_root() {
    let error = must_err(
        parse_qianji_server_args(["--flowhub-root", "  "]),
        "empty Flowhub root should be rejected",
    );

    assert!(
        error.contains("--flowhub-root must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn qianji_server_rejects_empty_control_ledger_path() {
    let error = must_err(
        parse_qianji_server_args(["--control-ledger", "  "]),
        "empty control ledger path should be rejected",
    );

    assert!(
        error.contains("--control-ledger must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn qianji_server_rejects_unknown_argument() {
    let error = must_err(
        parse_qianji_server_args(["--port", "38130"]),
        "unknown args should be rejected",
    );

    assert!(
        error.contains("unsupported qianji-server argument"),
        "unexpected error: {error}"
    );
}
