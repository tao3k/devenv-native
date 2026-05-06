use clap::Parser;
use std::path::PathBuf;
use xiuxian_wendao_client::{ClientCli, ClientCommand, SemanticCommand};

#[test]
fn parses_semantic_describe_read_model_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "semantic",
        "describe-read-model",
        "semantic/custom",
    ]);
    let ClientCommand::Semantic { command } = cli.command else {
        panic!("expected semantic command");
    };
    match command {
        SemanticCommand::DescribeReadModel(args) => {
            assert_eq!(args.path, Some(PathBuf::from("semantic/custom")));
        }
        SemanticCommand::CheckReadModelSnapshot(_)
        | SemanticCommand::PlanReadModelMaterialization(_)
        | SemanticCommand::PreflightReadModelMaterialization(_)
        | SemanticCommand::QueryReadModel(_)
        | SemanticCommand::RefreshProjections(_)
        | SemanticCommand::SnapshotReadModel(_) => panic!("expected describe read-model command"),
    }
}

#[test]
fn parses_semantic_snapshot_read_model_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "semantic",
        "snapshot-read-model",
        "semantic/custom",
    ]);
    let ClientCommand::Semantic { command } = cli.command else {
        panic!("expected semantic command");
    };
    match command {
        SemanticCommand::SnapshotReadModel(args) => {
            assert_eq!(args.path, Some(PathBuf::from("semantic/custom")));
        }
        SemanticCommand::CheckReadModelSnapshot(_)
        | SemanticCommand::DescribeReadModel(_)
        | SemanticCommand::PlanReadModelMaterialization(_)
        | SemanticCommand::PreflightReadModelMaterialization(_)
        | SemanticCommand::QueryReadModel(_)
        | SemanticCommand::RefreshProjections(_) => panic!("expected snapshot read-model command"),
    }
}

#[test]
fn parses_semantic_check_read_model_snapshot_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "semantic",
        "check-read-model-snapshot",
        "--expect",
        "blake3:0000000000000000000000000000000000000000000000000000000000000000",
        "semantic/custom",
    ]);
    let ClientCommand::Semantic { command } = cli.command else {
        panic!("expected semantic command");
    };
    match command {
        SemanticCommand::CheckReadModelSnapshot(args) => {
            assert_eq!(
                args.expected_snapshot_revision,
                "blake3:0000000000000000000000000000000000000000000000000000000000000000"
            );
            assert_eq!(args.path, Some(PathBuf::from("semantic/custom")));
        }
        SemanticCommand::DescribeReadModel(_)
        | SemanticCommand::PlanReadModelMaterialization(_)
        | SemanticCommand::PreflightReadModelMaterialization(_)
        | SemanticCommand::QueryReadModel(_)
        | SemanticCommand::RefreshProjections(_)
        | SemanticCommand::SnapshotReadModel(_) => {
            panic!("expected check read-model snapshot command")
        }
    }
}

#[test]
fn parses_semantic_plan_read_model_materialization_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "semantic",
        "plan-read-model-materialization",
        "--expect-snapshot",
        "blake3:0000000000000000000000000000000000000000000000000000000000000000",
        "semantic/custom",
    ]);
    let ClientCommand::Semantic { command } = cli.command else {
        panic!("expected semantic command");
    };
    match command {
        SemanticCommand::PlanReadModelMaterialization(args) => {
            assert_eq!(
                args.expected_snapshot_revision.as_deref(),
                Some("blake3:0000000000000000000000000000000000000000000000000000000000000000")
            );
            assert_eq!(args.path, Some(PathBuf::from("semantic/custom")));
        }
        SemanticCommand::CheckReadModelSnapshot(_)
        | SemanticCommand::DescribeReadModel(_)
        | SemanticCommand::PreflightReadModelMaterialization(_)
        | SemanticCommand::QueryReadModel(_)
        | SemanticCommand::RefreshProjections(_)
        | SemanticCommand::SnapshotReadModel(_) => {
            panic!("expected plan read-model materialization command")
        }
    }
}

#[test]
fn parses_semantic_preflight_read_model_materialization_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "semantic",
        "preflight-read-model-materialization",
        "--expect-snapshot",
        "blake3:0000000000000000000000000000000000000000000000000000000000000000",
        "semantic/custom",
    ]);
    let ClientCommand::Semantic { command } = cli.command else {
        panic!("expected semantic command");
    };
    match command {
        SemanticCommand::PreflightReadModelMaterialization(args) => {
            assert_eq!(
                args.expected_snapshot_revision.as_deref(),
                Some("blake3:0000000000000000000000000000000000000000000000000000000000000000")
            );
            assert_eq!(args.path, Some(PathBuf::from("semantic/custom")));
        }
        SemanticCommand::CheckReadModelSnapshot(_)
        | SemanticCommand::DescribeReadModel(_)
        | SemanticCommand::PlanReadModelMaterialization(_)
        | SemanticCommand::QueryReadModel(_)
        | SemanticCommand::RefreshProjections(_)
        | SemanticCommand::SnapshotReadModel(_) => {
            panic!("expected preflight read-model materialization command")
        }
    }
}

#[test]
fn parses_semantic_query_read_model_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "semantic",
        "query-read-model",
        "--query",
        "select id from semantic_objects",
        "semantic/custom",
    ]);
    let ClientCommand::Semantic { command } = cli.command else {
        panic!("expected semantic command");
    };
    match command {
        SemanticCommand::QueryReadModel(args) => {
            assert_eq!(args.query_text, "select id from semantic_objects");
            assert_eq!(args.path, Some(PathBuf::from("semantic/custom")));
        }
        SemanticCommand::CheckReadModelSnapshot(_)
        | SemanticCommand::DescribeReadModel(_)
        | SemanticCommand::PlanReadModelMaterialization(_)
        | SemanticCommand::PreflightReadModelMaterialization(_)
        | SemanticCommand::RefreshProjections(_)
        | SemanticCommand::SnapshotReadModel(_) => panic!("expected query read-model command"),
    }
}
