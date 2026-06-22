use std::path::PathBuf;

use crate::qianji_server_cli::cli::{
    QianjiServerCommand, QianjiServerServeCommand, parse_qianji_server_args,
};

use crate::qianji_server_cli::tests::support::{must_ok, must_parse_addr};

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
            flight_bind_addr: None,
            valkey_url: None,
            require_valkey_ready: None,
            flowhub_root: None,
            control_ledger_path: None,
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
            flight_bind_addr: None,
            valkey_url: None,
            require_valkey_ready: None,
            flowhub_root: None,
            control_ledger_path: None,
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
            flight_bind_addr: None,
            valkey_url: None,
            require_valkey_ready: None,
            flowhub_root: None,
            control_ledger_path: None,
        })
    );
}

#[test]
fn qianji_server_accepts_custom_flight_bind_address() {
    let command = must_ok(
        parse_qianji_server_args(["--flight-bind", "127.0.0.1:0"]),
        "custom Flight bind address should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            flight_bind_addr: Some(must_parse_addr("127.0.0.1:0")),
            valkey_url: None,
            require_valkey_ready: None,
            flowhub_root: None,
            control_ledger_path: None,
        })
    );
}

#[test]
fn qianji_server_accepts_equals_flight_bind_address() {
    let command = must_ok(
        parse_qianji_server_args(["--flight-bind=127.0.0.1:0"]),
        "equals Flight bind address should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            flight_bind_addr: Some(must_parse_addr("127.0.0.1:0")),
            valkey_url: None,
            require_valkey_ready: None,
            flowhub_root: None,
            control_ledger_path: None,
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
            flight_bind_addr: None,
            valkey_url: Some("redis://127.0.0.1:6380/0".to_string()),
            require_valkey_ready: None,
            flowhub_root: None,
            control_ledger_path: None,
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
            flight_bind_addr: None,
            valkey_url: Some("redis://127.0.0.1:6381/0".to_string()),
            require_valkey_ready: None,
            flowhub_root: None,
            control_ledger_path: None,
        })
    );
}

#[test]
fn qianji_server_accepts_custom_flowhub_root() {
    let command = must_ok(
        parse_qianji_server_args(["--flowhub-root", "qianji-flowhub"]),
        "custom Flowhub root should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            flight_bind_addr: None,
            valkey_url: None,
            require_valkey_ready: None,
            flowhub_root: Some(PathBuf::from("qianji-flowhub")),
            control_ledger_path: None,
        })
    );
}

#[test]
fn qianji_server_accepts_equals_flowhub_root() {
    let command = must_ok(
        parse_qianji_server_args(["--flowhub-root=/tmp/qianji-flowhub"]),
        "equals Flowhub root should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            flight_bind_addr: None,
            valkey_url: None,
            require_valkey_ready: None,
            flowhub_root: Some(PathBuf::from("/tmp/qianji-flowhub")),
            control_ledger_path: None,
        })
    );
}

#[test]
fn qianji_server_accepts_custom_control_ledger_path() {
    let command = must_ok(
        parse_qianji_server_args(["--control-ledger", ".cache/qianji/control.duckdb"]),
        "custom control ledger path should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            flight_bind_addr: None,
            valkey_url: None,
            require_valkey_ready: None,
            flowhub_root: None,
            control_ledger_path: Some(PathBuf::from(".cache/qianji/control.duckdb")),
        })
    );
}

#[test]
fn qianji_server_accepts_equals_control_ledger_path() {
    let command = must_ok(
        parse_qianji_server_args(["--control-ledger=/tmp/qianji-control.duckdb"]),
        "equals control ledger path should parse",
    );

    assert_eq!(
        command,
        QianjiServerCommand::Serve(QianjiServerServeCommand {
            bind_addr: None,
            flight_bind_addr: None,
            valkey_url: None,
            require_valkey_ready: None,
            flowhub_root: None,
            control_ledger_path: Some(PathBuf::from("/tmp/qianji-control.duckdb")),
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
            flight_bind_addr: None,
            valkey_url: None,
            require_valkey_ready: Some(true),
            flowhub_root: None,
            control_ledger_path: None,
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
            flight_bind_addr: None,
            valkey_url: None,
            require_valkey_ready: Some(false),
            flowhub_root: None,
            control_ledger_path: None,
        })
    );
}
