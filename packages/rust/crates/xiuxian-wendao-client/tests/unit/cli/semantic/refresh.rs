use clap::Parser;
use std::path::PathBuf;
use xiuxian_wendao_client::{ClientCli, ClientCommand, SemanticCommand};

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
        SemanticCommand::CheckReadModelSnapshot(_)
        | SemanticCommand::DescribeReadModel(_)
        | SemanticCommand::PlanReadModelMaterialization(_)
        | SemanticCommand::PreflightReadModelMaterialization(_)
        | SemanticCommand::QueryReadModel(_)
        | SemanticCommand::SnapshotReadModel(_) => panic!("expected refresh projections command"),
        SemanticCommand::RefreshProjections(args) => {
            assert_eq!(args.paths, vec![PathBuf::from("semantic/custom")]);
            assert_eq!(args.interval_secs, 0);
            assert!(args.max_runs.is_none());
            assert!(!args.require_clean_worktree);
        }
    }
}

#[test]
fn parses_semantic_refresh_projections_runner_options() {
    let cli = ClientCli::parse_from([
        "wendao",
        "semantic",
        "refresh-projections",
        "--interval-secs",
        "300",
        "--max-runs",
        "3",
        "--require-clean-worktree",
        "semantic/custom",
    ]);
    let ClientCommand::Semantic { command } = cli.command else {
        panic!("expected semantic command");
    };
    match command {
        SemanticCommand::CheckReadModelSnapshot(_)
        | SemanticCommand::DescribeReadModel(_)
        | SemanticCommand::PlanReadModelMaterialization(_)
        | SemanticCommand::PreflightReadModelMaterialization(_)
        | SemanticCommand::QueryReadModel(_)
        | SemanticCommand::SnapshotReadModel(_) => panic!("expected refresh projections command"),
        SemanticCommand::RefreshProjections(args) => {
            assert_eq!(args.paths, vec![PathBuf::from("semantic/custom")]);
            assert_eq!(args.interval_secs, 300);
            assert_eq!(args.max_runs.map(std::num::NonZeroUsize::get), Some(3));
            assert!(args.require_clean_worktree);
        }
    }
}
