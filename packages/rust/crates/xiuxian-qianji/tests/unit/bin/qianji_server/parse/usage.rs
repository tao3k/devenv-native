use crate::qianji_server_cli::cli::{
    QianjiServerCommand, parse_qianji_server_args, qianji_server_usage,
};

use crate::qianji_server_cli::tests::support::must_ok;

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
        usage.contains("--flowhub-root <path>"),
        "usage should document explicit Flowhub root binding: {usage}"
    );
    assert!(
        usage.contains("--control-ledger <path>"),
        "usage should document control-ledger activity evidence: {usage}"
    );
    assert!(
        !usage.contains(&removed_backend) && !usage.contains(&removed_backend_lower),
        "usage should not mention removed local checkpoint support: {usage}"
    );
}
