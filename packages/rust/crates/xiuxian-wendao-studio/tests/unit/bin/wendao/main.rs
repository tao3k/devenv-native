use super::command_requires_index;
use crate::bin_support::wendao::types::{Cli, Command};
use clap::Parser;

#[test]
fn audit_template_command_does_not_require_link_graph_index() {
    let cli = Cli::parse_from(["wendao", "audit", "--template", "johnny-decimal"]);

    assert!(!command_requires_index(&cli.command));
}

#[test]
fn ordinary_audit_command_requires_link_graph_index() {
    let cli = Cli::parse_from(["wendao", "audit", "docs"]);

    assert!(command_requires_index(&cli.command));
    assert!(matches!(cli.command, Command::Audit(_)));
}
