use super::{can_execute_immediate, client_context_from_cli};
use crate::bin_support::wendao::types::Cli;
use clap::Parser;
use xiuxian_wendao_client::OutputFormat as ClientOutputFormat;

#[test]
fn embedded_client_context_preserves_json_output() {
    let cli = Cli::parse_from([
        "wendao",
        "--output",
        "json",
        "lint",
        "markdown",
        "README.md",
    ]);

    assert_eq!(
        client_context_from_cli(&cli).output(),
        ClientOutputFormat::Json
    );
}

#[test]
fn embedded_client_context_defaults_get_output_to_text() {
    let cli = Cli::parse_from(["wendao", "get", "toc", "README.md"]);

    assert_eq!(
        client_context_from_cli(&cli).output(),
        ClientOutputFormat::Text
    );
}

#[test]
fn embedded_client_context_preserves_pretty_output() {
    let cli = Cli::parse_from([
        "wendao",
        "--output",
        "pretty",
        "lint",
        "markdown",
        "README.md",
    ]);

    assert_eq!(
        client_context_from_cli(&cli).output(),
        ClientOutputFormat::Pretty
    );
}

#[test]
fn embedded_client_context_preserves_config_path() {
    let cli = Cli::parse_from(["wendao", "--conf", "wendao.toml", "get", "toc", "docs"]);

    assert!(client_context_from_cli(&cli).config_file().is_some());
}

#[test]
fn audit_template_can_execute_without_async_runtime() {
    let cli = Cli::parse_from(["wendao", "audit", "--template", "johnny-decimal"]);

    assert!(can_execute_immediate(&cli.command));
}

#[test]
fn ordinary_audit_still_uses_runtime_path() {
    let cli = Cli::parse_from(["wendao", "audit", "docs"]);

    assert!(!can_execute_immediate(&cli.command));
}
