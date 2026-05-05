use std::sync::Arc;

use crate::contracts::UiProjectConfig;
use crate::studio::router::handlers::graph::neighbors::graph_neighbors;
use crate::studio::router::handlers::graph::shared::GraphNeighborsQuery;
use crate::studio::router::handlers::graph::tests::{
    assert_graph_neighbors_include_link_target, assert_graph_neighbors_include_path, build_fixture,
    build_fixture_with_projects, graph_neighbors_response, graph_neighbors_snapshot_payload,
};
use crate::studio::test_support::assert_studio_json_snapshot;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::StatusCode;

#[tokio::test]
async fn graph_neighbors_returns_center_nodes_and_links() {
    let fixture = build_fixture(&[
        ("alpha.md", "# Alpha\n\nSee [[beta]].\n"),
        ("beta.md", "# Beta\n\nBody.\n"),
    ]);

    let response = graph_neighbors_response(&fixture, "kernel/alpha.md", 2, 20).await;

    assert_eq!(response.center.id, "kernel/alpha.md");
    assert!(
        response
            .nodes
            .iter()
            .any(|node| node.id == "kernel/alpha.md")
    );
    assert!(
        response
            .nodes
            .iter()
            .any(|node| node.id == "kernel/beta.md")
    );
    assert!(
        response
            .links
            .iter()
            .any(|link| { link.source == "kernel/alpha.md" && link.target == "kernel/beta.md" })
    );
    assert!(response.total_nodes >= 2);
    assert!(response.total_links >= 1);
}

#[tokio::test]
async fn graph_neighbors_resolves_relative_markdown_links_from_index_pages() {
    let fixture = build_fixture(&[
        (
            "docs/index.md",
            concat!(
                "# Documentation Index\n\n",
                "This file is the top-level entry for major documentation tracks.\n\n",
                "## Testing\n\n",
                "- [Testing Documentation](testing/README.md)\n",
            ),
        ),
        (
            "docs/testing/README.md",
            "# Testing Documentation\n\nBody.\n",
        ),
    ]);

    let response = graph_neighbors_response(&fixture, "kernel/docs/index.md", 1, 20).await;

    assert!(
        response.total_nodes >= 2,
        "expected docs/index.md to surface related documentation nodes, got {}",
        response.total_nodes
    );
    assert!(
        response.total_links >= 1,
        "expected docs/index.md to surface outbound graph edges, got {}",
        response.total_links
    );
    assert_graph_neighbors_include_path(&response, "testing/README.md");
    assert_graph_neighbors_include_link_target(&response, "testing/README.md");

    assert_studio_json_snapshot(
        "graph_neighbors_index_page_links_payload",
        graph_neighbors_snapshot_payload(response),
    );
}

#[tokio::test]
async fn graph_neighbors_returns_not_found_for_missing_node() {
    let fixture = build_fixture(&[("alpha.md", "# Alpha\n\nBody.\n")]);

    let Err(error) = graph_neighbors(
        State(Arc::clone(&fixture.state)),
        AxumPath("missing.md".to_string()),
        Query(GraphNeighborsQuery {
            direction: None,
            hops: None,
            limit: None,
        }),
    )
    .await
    else {
        panic!("missing graph node should fail");
    };

    assert_eq!(error.status(), StatusCode::NOT_FOUND);
    assert_eq!(error.code(), "NOT_FOUND");
}

#[tokio::test]
async fn graph_neighbors_resolves_project_prefixed_display_paths() {
    let fixture = build_fixture(&[
        ("docs/alpha.md", "# Alpha\n\nSee [[beta]].\n"),
        ("docs/beta.md", "# Beta\n\nBody.\n"),
    ]);

    let response = graph_neighbors(
        State(Arc::clone(&fixture.state)),
        AxumPath("kernel/docs/alpha.md".to_string()),
        Query(GraphNeighborsQuery {
            direction: Some("both".to_string()),
            hops: Some(1),
            limit: Some(20),
        }),
    )
    .await
    .unwrap_or_else(|error| panic!("display-path graph neighbors should succeed: {error:?}"))
    .0;

    assert_eq!(response.center.id, "kernel/docs/alpha.md");
    assert!(
        response
            .nodes
            .iter()
            .any(|node| node.id == "kernel/docs/beta.md")
    );
}

#[tokio::test]
async fn graph_neighbors_prefers_exact_display_path_for_project_scoped_index_pages() {
    let fixture = build_fixture_with_projects(
        &[
            (
                "frontend/docs/index.md",
                concat!(
                    "---\n",
                    "title: Qianji Studio DocOS Kernel: Map of Content\n",
                    "---\n\n",
                    "# Qianji Studio DocOS Kernel: Map of Content\n\n",
                    "- [[chapter]]\n",
                ),
            ),
            ("frontend/docs/chapter.md", "# Kernel Chapter\n\nBody.\n"),
            (
                "docs/index.md",
                concat!(
                    "---\n",
                    "title: Documentation Index\n",
                    "---\n\n",
                    "# Documentation Index\n\n",
                    "Body.\n",
                ),
            ),
        ],
        &[
            UiProjectConfig {
                name: "kernel".to_string(),
                root: "frontend".to_string(),
                dirs: vec!["docs".to_string()],
            },
            UiProjectConfig {
                name: "main".to_string(),
                root: ".".to_string(),
                dirs: vec!["docs".to_string()],
            },
        ],
    );

    let response = graph_neighbors(
        State(Arc::clone(&fixture.state)),
        AxumPath("kernel/docs/index.md".to_string()),
        Query(GraphNeighborsQuery {
            direction: Some("both".to_string()),
            hops: Some(1),
            limit: Some(20),
        }),
    )
    .await
    .unwrap_or_else(|error| panic!("project-scoped graph neighbors should succeed: {error:?}"))
    .0;

    assert_eq!(
        response.center.label,
        "Qianji Studio DocOS Kernel: Map of Content"
    );
    assert!(
        response
            .nodes
            .iter()
            .any(|node| node.id == "kernel/docs/chapter.md"),
        "expected kernel docs chapter to be present in graph neighbors"
    );
    assert!(
        response
            .nodes
            .iter()
            .all(|node| !node.id.starts_with("main/docs/") || node.id == "main/docs/index.md"),
        "expected project-scoped lookup to stay on the kernel document"
    );

    assert_studio_json_snapshot(
        "graph_neighbors_project_scoped_display_path_payload",
        graph_neighbors_snapshot_payload(response),
    );
}
