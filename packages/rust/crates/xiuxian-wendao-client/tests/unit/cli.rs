use clap::Parser;
use std::path::PathBuf;
use xiuxian_wendao_client::{ClientCli, ClientCommand, GetCommand, LintCommand, SemanticCommand};

#[test]
fn parses_markdown_lint_command() {
    let cli = ClientCli::parse_from(["wendao", "lint", "markdown", "docs"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    assert!(matches!(command, LintCommand::Markdown(_)));
}

#[test]
fn parses_semantic_lint_command() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert!(!args.validation.semantic_sql_guard);
            assert!(args.paths.is_empty());
        }
        LintCommand::Markdown(_) => panic!("expected semantic lint command"),
    }
}

#[test]
fn parses_semantic_lint_explicit_path() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic", "semantic"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert_eq!(args.paths, vec![PathBuf::from("semantic")]);
        }
        LintCommand::Markdown(_) => panic!("expected semantic lint command"),
    }
}

#[test]
fn parses_semantic_lint_sql_guard_flag() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic", "--semantic-sql-guard"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert!(args.validation.semantic_sql_guard);
            assert!(args.paths.is_empty());
        }
        LintCommand::Markdown(_) => panic!("expected semantic lint command"),
    }
}

#[test]
fn parses_semantic_lint_refresh_projections_flag() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic", "--refresh-projections"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert!(args.writeback.refresh_projections);
            assert!(args.paths.is_empty());
        }
        LintCommand::Markdown(_) => panic!("expected semantic lint command"),
    }
}

#[test]
fn parses_semantic_lint_lifecycle_plan_flag() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic", "--lifecycle-plan"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert!(args.validation.lifecycle_plan);
            assert!(args.paths.is_empty());
        }
        LintCommand::Markdown(_) => panic!("expected semantic lint command"),
    }
}

#[test]
fn parses_semantic_lint_projection_refresh_plan_flag() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic", "--projection-refresh-plan"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert!(args.validation.projection.projection_refresh_plan);
            assert!(args.paths.is_empty());
        }
        LintCommand::Markdown(_) => panic!("expected semantic lint command"),
    }
}

#[test]
fn parses_semantic_lint_require_fresh_projections_flag() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic", "--require-fresh-projections"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert!(args.validation.projection.require_fresh_projections);
            assert!(args.paths.is_empty());
        }
        LintCommand::Markdown(_) => panic!("expected semantic lint command"),
    }
}

#[test]
fn parses_semantic_lint_apply_lifecycle_plan_flag() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic", "--apply-lifecycle-plan"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert!(args.writeback.apply_lifecycle_plan);
            assert!(args.paths.is_empty());
        }
        LintCommand::Markdown(_) => panic!("expected semantic lint command"),
    }
}

#[test]
fn parses_semantic_refresh_projections_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "semantic",
        "refresh-projections",
        "semantic/custom",
    ]);
    let ClientCommand::Semantic { command } = cli.command else {
        panic!("expected semantic command");
    };
    match command {
        SemanticCommand::RefreshProjections(args) => {
            assert_eq!(args.paths, vec![PathBuf::from("semantic/custom")]);
        }
    }
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
