use clap::Parser;
use xiuxian_wendao_client::{ClientCli, ClientCommand, LintCommand};

#[test]
fn parses_markdown_lint_command() {
    let cli = ClientCli::parse_from(["wendao", "lint", "markdown", "docs"]);
    let ClientCommand::Lint { command } = cli.command;
    assert!(matches!(command, LintCommand::Markdown(_)));
}
