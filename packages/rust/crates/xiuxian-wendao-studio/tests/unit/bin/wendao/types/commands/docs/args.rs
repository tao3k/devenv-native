use super::{
    DocsNavigationArgs, DocsNodeArgs, DocsPageArgs, DocsPageIndexArgs, DocsPageIndexOutlineArgs,
    DocsSearchArgs, DocsSearchPageIndexArgs, DocsSegmentArgs, DocsTocArgs, ProjectionPageKindArg,
};

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
fn docs_page_index_args_capture_repo() {
    let args = DocsPageIndexArgs {
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
fn docs_search_page_index_args_capture_query_filter_and_limit() {
    let args = DocsSearchPageIndexArgs {
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
fn docs_page_index_outline_args_capture_repo_and_page_id() {
    let args = DocsPageIndexOutlineArgs {
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
