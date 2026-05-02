use super::{
    Command, DocsCommand, DocsNodeArgs, DocsPageArgs, DocsPageIndexArgs, DocsPageIndexOutlineArgs,
    DocsSearchArgs, DocsSearchPageIndexArgs, DocsSegmentArgs, DocsTocArgs, ProjectionPageKindArg,
    docs,
};

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
            | DocsCommand::PageIndexOutline(_)
            | DocsCommand::PageIndex(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchPageIndex(_)
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
fn docs_command_creation_wraps_page_index_outline_variant() {
    let command = docs(DocsCommand::PageIndexOutline(DocsPageIndexOutlineArgs {
        repo: "projectionica".to_string(),
        page_id: "page-id".to_string(),
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::PageIndexOutline(args) => {
                assert_eq!(args.repo, "projectionica");
                assert_eq!(args.page_id, "page-id");
            }
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::PageIndex(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchPageIndex(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs page-index-outline variant"),
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}

#[test]
fn docs_command_creation_wraps_page_index_variant() {
    let command = docs(DocsCommand::PageIndex(DocsPageIndexArgs {
        repo: "projectionica".to_string(),
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::PageIndex(args) => assert_eq!(args.repo, "projectionica"),
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::PageIndexOutline(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchPageIndex(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => {
                panic!("expected docs page-index command");
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
            | DocsCommand::PageIndexOutline(_)
            | DocsCommand::PageIndex(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchPageIndex(_)
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
            | DocsCommand::PageIndexOutline(_)
            | DocsCommand::PageIndex(_)
            | DocsCommand::Segment(_)
            | DocsCommand::SearchPageIndex(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs search command"),
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}

#[test]
fn docs_command_creation_wraps_search_page_index_variant() {
    let command = docs(DocsCommand::SearchPageIndex(DocsSearchPageIndexArgs {
        repo: "projectionica".to_string(),
        query: "anchors".to_string(),
        kind: Some(ProjectionPageKindArg::Reference),
        limit: 3,
    }));

    match command {
        Command::Docs { command } => match command {
            DocsCommand::SearchPageIndex(args) => {
                assert_eq!(args.repo, "projectionica");
                assert_eq!(args.query, "anchors");
                assert_eq!(args.kind, Some(ProjectionPageKindArg::Reference));
                assert_eq!(args.limit, 3);
            }
            DocsCommand::Page(_)
            | DocsCommand::Tree(_)
            | DocsCommand::PageIndexOutline(_)
            | DocsCommand::PageIndex(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::Node(_)
            | DocsCommand::Toc(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs search-page-index variant"),
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
            | DocsCommand::PageIndexOutline(_)
            | DocsCommand::PageIndex(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchPageIndex(_)
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
            | DocsCommand::PageIndexOutline(_)
            | DocsCommand::PageIndex(_)
            | DocsCommand::Segment(_)
            | DocsCommand::Search(_)
            | DocsCommand::SearchPageIndex(_)
            | DocsCommand::Node(_)
            | DocsCommand::Navigation(_)
            | DocsCommand::Context(_) => panic!("expected docs toc command"),
        },
        other => panic!("expected docs command, got {other:?}"),
    }
}
