use super::{
    CanonicalScopeTarget, DocsPageIndexDocumentsResult, DocsPageIndexTreesResult,
    ProjectedPageIndexDocument, ProjectedPageIndexLink, ProjectedPageIndexNode,
    ProjectedPageIndexSection, ProjectedPageIndexTree, ProjectionPageKind, ScopeTargetKind,
    build_local_page_index_trees, build_local_toc_documents, render_page_index_markdown,
    render_toc_markdown,
};
use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

#[test]
fn get_command_builds_local_toc_documents_from_file_target() {
    let temp_dir = tempdir_or_panic();
    let file_path = write_markdown(
        temp_dir.path(),
        "docs/guides/intro.md",
        "# Intro\n\n## Usage\n\nUse the thing.\n",
    );
    let scope = CanonicalScopeTarget {
        path: fs::canonicalize(&file_path)
            .unwrap_or_else(|error| panic!("canonical file: {error}")),
        kind: ScopeTargetKind::File,
    };

    let result = build_local_toc_documents(&scope, temp_dir.path())
        .unwrap_or_else(|error| panic!("local toc build should succeed: {error}"));

    assert_eq!(result.repo_id, "local");
    assert_eq!(result.documents.len(), 1);
    assert_eq!(result.documents[0].path, display_test_path(&file_path));
    assert_eq!(result.documents[0].doc_id, "docs/guides/intro.md");
    assert_eq!(result.documents[0].page_id, "local:docs/guides/intro.md");
    assert_eq!(result.documents[0].sections.len(), 2);
}

#[test]
fn get_command_builds_local_page_index_trees_from_directory_target() {
    let temp_dir = tempdir_or_panic();
    write_markdown(
        temp_dir.path(),
        "docs/guides/intro.md",
        "# Intro\n\n## Usage\n\nUse the thing.\n",
    );
    write_markdown(
        temp_dir.path(),
        "docs/reference/api.md",
        "# API\n\n## Endpoint\n\n`GET /api`.\n",
    );
    write_markdown(
        temp_dir.path(),
        "docs/reference/notes.txt",
        "plain text should not be included\n",
    );
    let scope = CanonicalScopeTarget {
        path: fs::canonicalize(temp_dir.path().join("docs"))
            .unwrap_or_else(|error| panic!("canonical dir: {error}")),
        kind: ScopeTargetKind::Directory,
    };

    let result = build_local_page_index_trees(&scope, temp_dir.path())
        .unwrap_or_else(|error| panic!("local page-index build should succeed: {error}"));

    let paths = result
        .trees
        .iter()
        .map(|tree| tree.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(result.repo_id, "local");
    assert_eq!(
        paths,
        vec![
            display_test_path(temp_dir.path().join("docs/guides/intro.md").as_path()),
            display_test_path(temp_dir.path().join("docs/reference/api.md").as_path()),
        ]
    );
    assert!(result.trees.iter().all(|tree| tree.root_count > 0));
}

#[test]
fn get_command_builds_local_toc_documents_from_directory_target_in_path_order() {
    let temp_dir = tempdir_or_panic();
    write_markdown(
        temp_dir.path(),
        "docs/z-last.md",
        "# Z Last\n\n## End\n\nLast document.\n",
    );
    write_markdown(
        temp_dir.path(),
        "docs/a-first.md",
        "# A First\n\n## Start\n\nFirst document.\n",
    );
    let scope = CanonicalScopeTarget {
        path: fs::canonicalize(temp_dir.path().join("docs"))
            .unwrap_or_else(|error| panic!("canonical dir: {error}")),
        kind: ScopeTargetKind::Directory,
    };

    let result = build_local_toc_documents(&scope, temp_dir.path())
        .unwrap_or_else(|error| panic!("local toc build should succeed: {error}"));

    let paths = result
        .documents
        .iter()
        .map(|document| document.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec![
            display_test_path(temp_dir.path().join("docs/a-first.md").as_path()),
            display_test_path(temp_dir.path().join("docs/z-last.md").as_path()),
        ]
    );
}

#[test]
fn get_command_skips_hidden_project_runtime_dirs_by_default() {
    let temp_dir = tempdir_or_panic();
    write_markdown(
        temp_dir.path(),
        "docs/guide.md",
        "# Guide\n\n## Usage\n\nVisible document.\n",
    );
    write_markdown(
        temp_dir.path(),
        ".cache/generated.md",
        "# Generated\n\n## Cache\n\nShould stay hidden.\n",
    );
    write_markdown(
        temp_dir.path(),
        ".data/persisted.md",
        "# Persisted\n\n## Data\n\nShould stay hidden.\n",
    );
    let scope = CanonicalScopeTarget {
        path: fs::canonicalize(temp_dir.path())
            .unwrap_or_else(|error| panic!("canonical dir: {error}")),
        kind: ScopeTargetKind::Directory,
    };

    let result = build_local_toc_documents(&scope, temp_dir.path())
        .unwrap_or_else(|error| panic!("local toc build should succeed: {error}"));

    let paths = result
        .documents
        .iter()
        .map(|document| document.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec![display_test_path(
            temp_dir.path().join("docs/guide.md").as_path()
        )]
    );
}

#[test]
fn get_command_renders_toc_text_output_as_compact_markdown() {
    let path = display_test_path(Path::new("/tmp/README.md"));
    let result = DocsPageIndexDocumentsResult {
        repo_id: "local".to_string(),
        documents: vec![ProjectedPageIndexDocument {
            path: path.clone(),
            doc_id: "README.md".to_string(),
            title: "Demo".to_string(),
            sections: vec![
                ProjectedPageIndexSection {
                    heading_path: "Demo".to_string(),
                    title: "Demo".to_string(),
                    level: 1,
                    line_range: (1, 10),
                    attributes: Vec::new(),
                },
                ProjectedPageIndexSection {
                    heading_path: "Demo / Usage".to_string(),
                    title: "Usage".to_string(),
                    level: 2,
                    line_range: (11, 20),
                    attributes: Vec::new(),
                },
            ],
            ..ProjectedPageIndexDocument::default()
        }],
    };

    let rendered = render_toc_markdown(&result);

    assert!(rendered.starts_with(format!("path: {path}").as_str()));
    assert!(rendered.contains("title: Demo | sections: 2"));
    assert!(rendered.contains("# Demo -> [L1 1-10]"));
    assert!(rendered.contains("## Usage -> [L2 11-20]"));
    assert!(!rendered.contains("# TOC"));
    assert!(!rendered.contains("source-level"));
    assert!(!rendered.contains("[L1 1-10] Demo"));
    assert!(!rendered.contains("\"documents\""));
}

#[test]
fn get_command_renders_page_index_text_output_as_compact_markdown() {
    let path = display_test_path(Path::new("/tmp/README.md"));
    let result = DocsPageIndexTreesResult {
        repo_id: "local".to_string(),
        trees: vec![ProjectedPageIndexTree {
            path: path.clone(),
            doc_id: "README.md".to_string(),
            title: "Demo".to_string(),
            root_count: 1,
            kind: ProjectionPageKind::Explanation,
            roots: vec![ProjectedPageIndexNode {
                node_id: "README.md#demo".to_string(),
                title: "Demo".to_string(),
                level: 1,
                structural_path: vec!["Demo".to_string()],
                line_range: (1, 10),
                token_count: 2,
                is_thinned: false,
                text: "demo body".to_string(),
                summary: None,
                links: vec![ProjectedPageIndexLink {
                    kind: "wiki_link".to_string(),
                    target: "docs/guide.md#usage".to_string(),
                    surface: "[[docs/guide.md#usage|Guide]]".to_string(),
                }],
                children: vec![ProjectedPageIndexNode {
                    node_id: "README.md#usage".to_string(),
                    title: "Usage".to_string(),
                    level: 2,
                    structural_path: vec!["Demo".to_string(), "Usage".to_string()],
                    line_range: (11, 20),
                    token_count: 2,
                    is_thinned: false,
                    text: "usage body".to_string(),
                    summary: None,
                    links: vec![
                        ProjectedPageIndexLink {
                            kind: "markdown_link".to_string(),
                            target: "https://example.com".to_string(),
                            surface: "[API](https://example.com)".to_string(),
                        },
                        ProjectedPageIndexLink {
                            kind: "markdown_image".to_string(),
                            target: "assets/diagram.png".to_string(),
                            surface: "![Diagram](assets/diagram.png)".to_string(),
                        },
                    ],
                    children: Vec::new(),
                }],
            }],
            ..ProjectedPageIndexTree::default()
        }],
    };

    let rendered = render_page_index_markdown(&result);

    assert!(rendered.starts_with(format!("path: {path}").as_str()));
    assert!(rendered.contains("kind: Explanation | roots: 1 | nodes: 2 | links: 2 | embeds: 1"));
    assert!(rendered.contains("# Demo -> [L1 1-10]"));
    assert!(rendered.contains("links: [[docs/guide.md#usage|Guide]]"));
    assert!(rendered.contains("## Usage -> [L2 11-20]"));
    assert!(rendered.contains("links: [API](https://example.com)"));
    assert!(rendered.contains("embeds: ![Diagram](assets/diagram.png)"));
    assert!(!rendered.contains("markdown_link:"));
    assert!(!rendered.contains("wiki_link:"));
    assert!(!rendered.contains("# Page Index"));
    assert!(!rendered.contains("\"trees\""));
}

fn tempdir_or_panic() -> TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}

fn write_markdown(root: &Path, relative_path: &str, content: &str) -> PathBuf {
    let path = root.join(relative_path);
    let Some(parent) = path.parent() else {
        panic!("expected parent directory for test file");
    };
    fs::create_dir_all(parent).unwrap_or_else(|error| panic!("create dir: {error}"));
    fs::write(&path, content).unwrap_or_else(|error| panic!("write markdown: {error}"));
    path
}

fn display_test_path(path: &Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}
