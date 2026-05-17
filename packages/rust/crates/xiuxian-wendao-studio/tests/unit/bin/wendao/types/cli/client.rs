use super::{Cli, Command, Parser};

#[test]
fn parses_embedded_client_lint_command() {
    let cli = Cli::parse_from(["wendao", "lint", "markdown", "README.md"]);
    assert!(matches!(cli.command, Command::Client(_)));
}

#[test]
fn parses_get_toc_command() {
    let cli = Cli::parse_from(["wendao", "get", "toc", "docs/guides"]);
    assert!(matches!(cli.command, Command::Client(_)));
}

#[test]
fn parses_get_page_index_command() {
    let cli = Cli::parse_from(["wendao", "get", "page-index"]);
    assert!(matches!(cli.command, Command::Client(_)));
}
