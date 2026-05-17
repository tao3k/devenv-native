use super::{Cli, Command, Parser};

#[test]
fn parses_audit_load_episteme_command() {
    let cli = Cli::parse_from(["wendao", "audit", "--load", "wendao-episteme", "docs"]);

    let Command::Audit(args) = cli.command else {
        panic!("expected audit command");
    };

    assert_eq!(args.target, "docs");
    assert_eq!(args.load.as_deref(), Some("wendao-episteme"));
}

#[test]
fn parses_audit_template_command() {
    let cli = Cli::parse_from(["wendao", "audit", "--template", "johnny-decimal"]);

    let Command::Audit(args) = cli.command else {
        panic!("expected audit command");
    };

    assert_eq!(args.target, ".");
    assert_eq!(args.template.as_deref(), Some("johnny-decimal"));
}
