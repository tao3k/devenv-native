use super::support::{must_err, must_ok, must_parse_addr};
use crate::qianji_server_cli::cli::{
    QianjiServerCommand, QianjiServerServeCommand, parse_qianji_server_args, qianji_server_usage,
};

#[test]
fn qianji_server_leaves_default_bind_to_runtime_config() {
    let command = must_ok(
        parse_qianji_server_args(Vec::<String>::new()),
        "default qianji-server args should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            valkey_url: None,
            require_valkey_ready: None,
        })
    );
}

#[test]
fn qianji_server_accepts_custom_bind_address() {
    let command = must_ok(
        parse_qianji_server_args(["--bind", "127.0.0.1:0"]),
        "custom bind address should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: Some(must_parse_addr("127.0.0.1:0")),
            valkey_url: None,
            require_valkey_ready: None,
        })
    );
}

#[test]
fn qianji_server_accepts_equals_bind_address() {
    let command = must_ok(
        parse_qianji_server_args(["--bind=127.0.0.1:0"]),
        "equals bind address should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: Some(must_parse_addr("127.0.0.1:0")),
            valkey_url: None,
            require_valkey_ready: None,
        })
    );
}

#[test]
fn qianji_server_accepts_custom_valkey_url() {
    let command = must_ok(
        parse_qianji_server_args(["--valkey-url", "redis://127.0.0.1:6380/0"]),
        "custom valkey URL should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            valkey_url: Some("redis://127.0.0.1:6380/0".to_string()),
            require_valkey_ready: None,
        })
    );
}

#[test]
fn qianji_server_accepts_equals_valkey_url() {
    let command = must_ok(
        parse_qianji_server_args(["--valkey-url=redis://127.0.0.1:6381/0"]),
        "equals valkey URL should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            valkey_url: Some("redis://127.0.0.1:6381/0".to_string()),
            require_valkey_ready: None,
        })
    );
}

#[test]
fn qianji_server_accepts_require_valkey_ready() {
    let command = must_ok(
        parse_qianji_server_args(["--require-valkey-ready"]),
        "require Valkey readiness flag should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            valkey_url: None,
            require_valkey_ready: Some(true),
        })
    );
}

#[test]
fn qianji_server_accepts_no_require_valkey_ready() {
    let command = must_ok(
        parse_qianji_server_args(["--no-require-valkey-ready"]),
        "no require Valkey readiness flag should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            valkey_url: None,
            require_valkey_ready: Some(false),
        })
    );
}

#[test]
fn qianji_server_returns_help_command() {
    let command = must_ok(parse_qianji_server_args(["--help"]), "help should parse");

    assert_eq!(command, QianjiServerCommand::Help);
}

#[test]
fn qianji_server_usage_documents_valkey_only_http_checkpoints() {
    let usage = qianji_server_usage();
    let removed_backend = ["SQL", "ite"].concat();
    let removed_backend_lower = removed_backend.to_ascii_lowercase();

    assert!(
        usage.contains("HTTP checkpoint defaults are Valkey-only"),
        "usage should document Valkey-only HTTP checkpoints: {usage}"
    );
    assert!(
        !usage.contains(&removed_backend) && !usage.contains(&removed_backend_lower),
        "usage should not mention removed local checkpoint support: {usage}"
    );
}

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
