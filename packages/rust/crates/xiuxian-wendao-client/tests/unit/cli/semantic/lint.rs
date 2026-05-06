use clap::Parser;
use std::path::PathBuf;
use xiuxian_wendao_client::{ClientCli, ClientCommand, LintCommand};

#[test]
fn parses_semantic_lint_command() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert!(!args.validation.read_model_summary);
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
fn parses_semantic_lint_read_model_summary_flag() {
    let cli = ClientCli::parse_from(["wendao", "lint", "semantic", "--read-model-summary"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    match command {
        LintCommand::Semantic(args) => {
            assert!(args.validation.read_model_summary);
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
