use super::super::Command;
use super::*;
use crate::types::ProjectionPageKindArg;

#[test]
fn docs_page_args_capture_repo_and_page_id() {
    let args = DocsPageArgs {
        repo: "projectionica".to_string(),
        page_id: "repo:projectionica:projection:reference:doc:foo".to_string(),
    };

    assert_eq!(args.repo, "projectionica");
    assert_eq!(
        args.page_id,
        "repo:projectionica:projection:reference:doc:foo"
    );
}

#[test]
fn docs_toc_args_capture_repo() {
    let args = DocsTocArgs {
        repo: "projectionica".to_string(),
    };

    assert_eq!(args.repo, "projectionica");
}

#[test]
fn docs_structure_catalog_args_capture_repo() {
    let args = DocsStructureCatalogArgs {
        repo: "projectionica".to_string(),
    };

    assert_eq!(args.repo, "projectionica");
}

#[test]
fn docs_segment_args_capture_page_and_line_range() {
    let args = DocsSegmentArgs {
        repo: "projectionica".to_string(),
        page_id: "page-id".to_string(),
        line_start: 12,
        line_end: 18,
    };

    assert_eq!(args.repo, "projectionica");
    assert_eq!(args.page_id, "page-id");
    assert_eq!(args.line_start, 12);
    assert_eq!(args.line_end, 18);
}

#[test]
fn docs_node_args_capture_page_and_node_ids() {
    let args = DocsNodeArgs {
        repo: "projectionica".to_string(),
        page_id: "page-id".to_string(),
        node_id: "node-id".to_string(),
    };

    assert_eq!(args.repo, "projectionica");
    assert_eq!(args.page_id, "page-id");
    assert_eq!(args.node_id, "node-id");
}

#[test]
fn docs_search_args_capture_query_filter_and_limit() {
    let args = DocsSearchArgs {
        repo: "projectionica".to_string(),
        query: "solver".to_string(),
        kind: Some(ProjectionPageKindArg::Reference),
        limit: 4,
    };

    assert_eq!(args.repo, "projectionica");
    assert_eq!(args.query, "solver");
    assert_eq!(args.kind, Some(ProjectionPageKindArg::Reference));
    assert_eq!(args.limit, 4);
}

#[test]
fn docs_search_structure_args_capture_query_filter_and_limit() {
    let args = DocsSearchStructureArgs {
        repo: "projectionica".to_string(),
        query: "anchors".to_string(),
        kind: Some(ProjectionPageKindArg::Reference),
        limit: 3,
    };

    assert_eq!(args.repo, "projectionica");
    assert_eq!(args.query, "anchors");
    assert_eq!(args.kind, Some(ProjectionPageKindArg::Reference));
    assert_eq!(args.limit, 3);
}

#[test]
fn docs_tree_outline_args_capture_repo_and_page_id() {
    let args = DocsTreeOutlineArgs {
        repo: "projectionica".to_string(),
        page_id: "page-id".to_string(),
    };

    assert_eq!(args.repo, "projectionica");
    assert_eq!(args.page_id, "page-id");
}

#[test]
fn docs_navigation_args_capture_optional_context_fields() {
    let args = DocsNavigationArgs {
        repo: "projectionica".to_string(),
        page_id: "page-id".to_string(),
        node_id: Some("node-id".to_string()),
        family_kind: Some(ProjectionPageKindArg::Explanation),
        related_limit: 7,
        family_limit: 2,
    };

    assert_eq!(args.node_id.as_deref(), Some("node-id"));
    assert_eq!(args.family_kind, Some(ProjectionPageKindArg::Explanation));
    assert_eq!(args.related_limit, 7);
    assert_eq!(args.family_limit, 2);
}

#[test]
fn docs_command_creation_wraps_page_variant() {
    let command = docs(DocsCommand::Page(DocsPageArgs {
        repo: "projectionica".to_string(),
        page_id: "page-id".to_string(),
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::Page(args) => {
                assert_eq!(args.repo, "projectionica");
                assert_eq!(args.page_id, "page-id");
            }
            DocsCommand::Tree(_)
            | DocsCommand::TreeOutline(_)
            | DocsCommand::StructureCatalog(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchStructure(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => {
                panic!("expected docs page command");
            }
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}

#[test]
fn docs_command_creation_wraps_tree_outline_variant() {
    let command = docs(DocsCommand::TreeOutline(DocsTreeOutlineArgs {
        repo: "projectionica".to_string(),
        page_id: "page-id".to_string(),
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::TreeOutline(args) => {
                assert_eq!(args.repo, "projectionica");
                assert_eq!(args.page_id, "page-id");
            }
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::StructureCatalog(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchStructure(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs tree-outline command"),
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}

#[test]
fn docs_command_creation_wraps_structure_catalog_variant() {
    let command = docs(DocsCommand::StructureCatalog(DocsStructureCatalogArgs {
        repo: "projectionica".to_string(),
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::StructureCatalog(args) => assert_eq!(args.repo, "projectionica"),
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::TreeOutline(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchStructure(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => {
                panic!("expected docs structure-catalog command");
            }
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}

#[test]
fn docs_command_creation_wraps_segment_variant() {
    let command = docs(DocsCommand::Segment(DocsSegmentArgs {
        repo: "projectionica".to_string(),
        page_id: "page-id".to_string(),
        line_start: 12,
        line_end: 18,
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::Segment(args) => {
                assert_eq!(args.repo, "projectionica");
                assert_eq!(args.page_id, "page-id");
                assert_eq!(args.line_start, 12);
                assert_eq!(args.line_end, 18);
            }
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::TreeOutline(_)
            | DocsCommand::StructureCatalog(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchStructure(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs segment command"),
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}

#[test]
fn docs_command_creation_wraps_search_variant() {
    let command = docs(DocsCommand::Search(DocsSearchArgs {
        repo: "projectionica".to_string(),
        query: "solver".to_string(),
        kind: Some(ProjectionPageKindArg::Reference),
        limit: 4,
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::Search(args) => {
                assert_eq!(args.repo, "projectionica");
                assert_eq!(args.query, "solver");
                assert_eq!(args.kind, Some(ProjectionPageKindArg::Reference));
                assert_eq!(args.limit, 4);
            }
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::TreeOutline(_)
            | DocsCommand::StructureCatalog(_)
            | DocsCommand::Segment(_)
            | DocsCommand::SearchStructure(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs search command"),
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}

#[test]
fn docs_command_creation_wraps_search_structure_variant() {
    let command = docs(DocsCommand::SearchStructure(DocsSearchStructureArgs {
        repo: "projectionica".to_string(),
        query: "anchors".to_string(),
        kind: Some(ProjectionPageKindArg::Reference),
        limit: 3,
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::SearchStructure(args) => {
                assert_eq!(args.repo, "projectionica");
                assert_eq!(args.query, "anchors");
                assert_eq!(args.kind, Some(ProjectionPageKindArg::Reference));
                assert_eq!(args.limit, 3);
            }
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::TreeOutline(_)
            | DocsCommand::StructureCatalog(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs search-structure command"),
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}

#[test]
fn docs_command_creation_wraps_node_variant() {
    let command = docs(DocsCommand::Node(DocsNodeArgs {
        repo: "projectionica".to_string(),
        page_id: "page-id".to_string(),
        node_id: "node-id".to_string(),
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::Node(args) => {
                assert_eq!(args.repo, "projectionica");
                assert_eq!(args.page_id, "page-id");
                assert_eq!(args.node_id, "node-id");
            }
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::TreeOutline(_)
            | DocsCommand::StructureCatalog(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchStructure(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs node command"),
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}

#[test]
fn docs_command_creation_wraps_toc_variant() {
    let command = docs(DocsCommand::Toc(DocsTocArgs {
        repo: "projectionica".to_string(),
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::Toc(args) => assert_eq!(args.repo, "projectionica"),
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::TreeOutline(_)
            | DocsCommand::StructureCatalog(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchStructure(_)
            | DocsCommand::Node(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs toc command"),
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}
