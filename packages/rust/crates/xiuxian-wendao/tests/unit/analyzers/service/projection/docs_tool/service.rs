use super::{
    DocsNavigationOptions, DocsPageIndexTreeResult, DocsPageIndexTreesResult,
    DocsRetrievalContextOptions, DocsToolService, Path, PathBuf, ProjectedPageIndexNode,
    text_free_tree_result, text_free_trees_result,
};
use crate::analyzers::projection::{ProjectedPageIndexTree, ProjectionPageKind};

#[test]
fn docs_tool_service_starts_without_config_override() {
    let service = DocsToolService::from_project_root("/tmp/project", "repo-a");

    assert_eq!(service.project_root(), Path::new("/tmp/project"));
    assert_eq!(service.repo_id(), "repo-a");
    assert_eq!(service.config_path(), None);
}

#[test]
fn docs_tool_service_accepts_optional_config_override() {
    let config_path = PathBuf::from("/tmp/project/wendao.toml");
    let service = DocsToolService::from_project_root("/tmp/project", "repo-a")
        .with_optional_config_path(Some(config_path.clone()));

    assert_eq!(service.config_path(), Some(config_path.as_path()));
}

#[test]
fn navigation_options_default_to_docs_limits() {
    let options = DocsNavigationOptions::default();

    assert_eq!(options.related_limit, 5);
    assert_eq!(options.family_limit, 3);
    assert_eq!(options.node_id, None);
    assert_eq!(options.family_kind, None);
}

#[test]
fn navigation_options_normalize_zero_family_limit() {
    let options = DocsNavigationOptions {
        family_kind: Some(ProjectionPageKind::HowTo),
        family_limit: 0,
        ..DocsNavigationOptions::default()
    }
    .normalized();

    assert_eq!(options.family_limit, 1);
    assert_eq!(options.family_kind, Some(ProjectionPageKind::HowTo));
}

#[test]
fn retrieval_context_options_default_to_docs_limit() {
    let options = DocsRetrievalContextOptions::default();

    assert_eq!(options.related_limit, 5);
    assert_eq!(options.node_id, None);
}

#[test]
fn text_free_tree_result_clears_node_text_recursively() {
    let result = DocsPageIndexTreeResult {
        repo_id: "repo-a".to_string().into(),
        tree: Some(ProjectedPageIndexTree {
            repo_id: "repo-a".to_string().into(),
            page_id: "page-a".to_string().into(),
            kind: ProjectionPageKind::Reference,
            path: "reference/page-a.md".to_string().into(),
            doc_id: "doc:page-a".to_string().into(),
            title: "Page A".to_string(),
            root_count: 1,
            roots: vec![ProjectedPageIndexNode {
                node_id: "n1".to_string().into(),
                title: "Root".to_string(),
                level: 1,
                structural_path: vec!["Root".to_string()],
                line_range: (1, 3),
                token_count: 3,
                is_thinned: false,
                text: "root body".to_string(),
                summary: Some("summary".to_string()),
                children: vec![ProjectedPageIndexNode {
                    node_id: "n2".to_string().into(),
                    title: "Child".to_string(),
                    level: 2,
                    structural_path: vec!["Root".to_string(), "Child".to_string()],
                    line_range: (2, 3),
                    token_count: 2,
                    is_thinned: false,
                    text: "child body".to_string(),
                    summary: Some("child summary".to_string()),
                    children: Vec::new(),
                }],
            }],
        }),
    };

    let stripped = text_free_tree_result(result);
    let roots = stripped
        .tree
        .unwrap_or_else(|| panic!("expected stripped tree"))
        .roots;
    assert_eq!(roots[0].text, "");
    assert_eq!(roots[0].summary.as_deref(), Some("summary"));
    assert_eq!(roots[0].children[0].text, "");
    assert_eq!(
        roots[0].children[0].summary.as_deref(),
        Some("child summary")
    );
}

#[test]
fn text_free_trees_result_clears_node_text_recursively() {
    let result = DocsPageIndexTreesResult {
        repo_id: "repo-a".to_string().into(),
        trees: vec![ProjectedPageIndexTree {
            repo_id: "repo-a".to_string().into(),
            page_id: "page-a".to_string().into(),
            kind: ProjectionPageKind::Reference,
            path: "reference/page-a.md".to_string().into(),
            doc_id: "doc:page-a".to_string().into(),
            title: "Page A".to_string(),
            root_count: 1,
            roots: vec![ProjectedPageIndexNode {
                node_id: "n1".to_string().into(),
                title: "Root".to_string(),
                level: 1,
                structural_path: vec!["Root".to_string()],
                line_range: (1, 3),
                token_count: 3,
                is_thinned: false,
                text: "root body".to_string(),
                summary: Some("summary".to_string()),
                children: vec![ProjectedPageIndexNode {
                    node_id: "n2".to_string().into(),
                    title: "Child".to_string(),
                    level: 2,
                    structural_path: vec!["Root".to_string(), "Child".to_string()],
                    line_range: (2, 3),
                    token_count: 2,
                    is_thinned: false,
                    text: "child body".to_string(),
                    summary: Some("child summary".to_string()),
                    children: Vec::new(),
                }],
            }],
        }],
    };

    let stripped = text_free_trees_result(result);
    let roots = &stripped.trees[0].roots;
    assert_eq!(roots[0].text, "");
    assert_eq!(roots[0].summary.as_deref(), Some("summary"));
    assert_eq!(roots[0].children[0].text, "");
    assert_eq!(
        roots[0].children[0].summary.as_deref(),
        Some("child summary")
    );
}
