use super::client_context_from_cli;
use crate::types::Cli;
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
