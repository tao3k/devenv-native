use super::Cli;
use crate::types::Command;
use clap::Parser;

#[test]
fn parses_embedded_client_lint_command() {
    let cli = Cli::parse_from(["wendao", "lint", "markdown", "README.md"]);
    assert!(matches!(cli.command, Command::Client(_)));
}
