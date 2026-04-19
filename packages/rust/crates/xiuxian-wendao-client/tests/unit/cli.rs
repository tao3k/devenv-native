use clap::Parser;
use std::path::PathBuf;
use xiuxian_wendao_client::{ClientCli, ClientCommand, GetCommand, LintCommand};

#[test]
fn parses_markdown_lint_command() {
    let cli = ClientCli::parse_from(["wendao", "lint", "markdown", "docs"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    assert!(matches!(command, LintCommand::Markdown(_)));
}

#[test]
fn parses_get_toc_command() {
    let cli = ClientCli::parse_from(["wendao", "get", "toc", "docs/guides"]);
    let ClientCommand::Get { command } = cli.command else {
        panic!("expected get command");
    };
    match command {
        GetCommand::Toc(args) => assert_eq!(args.target, PathBuf::from("docs/guides")),
        GetCommand::PageIndex(_) => panic!("expected get toc command"),
    }
}

#[test]
fn parses_get_page_index_command() {
    let cli = ClientCli::parse_from(["wendao", "get", "page-index"]);
    let ClientCommand::Get { command } = cli.command else {
        panic!("expected get command");
    };
    match command {
        GetCommand::PageIndex(args) => assert_eq!(args.target, PathBuf::from(".")),
        GetCommand::Toc(_) => panic!("expected get page-index command"),
    }
}

#[test]
fn parses_get_toc_file_target_command() {
    let cli = ClientCli::parse_from(["wendao", "get", "toc", "docs/guides/intro.md"]);
    let ClientCommand::Get { command } = cli.command else {
        panic!("expected get command");
    };
    match command {
        GetCommand::Toc(args) => assert_eq!(args.target, PathBuf::from("docs/guides/intro.md")),
        GetCommand::PageIndex(_) => panic!("expected get toc command"),
    }
}

#[test]
fn parses_get_toc_command_with_repeatable_ignore_dirs() {
    let cli = ClientCli::parse_from([
        "wendao",
        "get",
        "toc",
        "docs",
        "--ignore",
        "generated",
        "--ignore",
        ".cache",
    ]);
    let ClientCommand::Get { command } = cli.command else {
        panic!("expected get command");
    };
    match command {
        GetCommand::Toc(args) => {
            assert_eq!(args.target, PathBuf::from("docs"));
            assert_eq!(
                args.ignore_dirs,
                vec!["generated".to_string(), ".cache".to_string()]
            );
        }
        GetCommand::PageIndex(_) => panic!("expected get toc command"),
    }
}
