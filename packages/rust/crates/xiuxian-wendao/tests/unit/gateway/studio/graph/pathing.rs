use super::support::{make_graph_fixture, push_ui_config_from_toml};
use super::*;

#[tokio::test]
async fn graph_neighbors_respects_glob_dir_filters() {
    let fixture = make_graph_fixture(vec![
        ("docs/public.md", "# Public\n\nSee [[private/index]].\n"),
        ("docs/private/index.md", "# Private\n\nBody.\n"),
    ]);
    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["docs", "**/*.md", "!docs/private/**"]
"#,
    );

    let blocked = graph_neighbors(
        fixture.state.as_ref(),
        "docs/private/index.md",
        "both",
        1,
        10,
    )
    .await;
    let Err(error) = blocked else {
        panic!("expected glob-filtered graph node to be hidden");
    };
    assert_eq!(error.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn graph_neighbors_resolves_vfs_alias_paths() {
    let fixture = make_graph_fixture(vec![
        ("packages/alpha/docs/index.md", "# Alpha\n\nBody.\n"),
        ("packages/beta/docs/index.md", "# Beta\n\nBody.\n"),
    ]);
    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.alpha]
root = "packages/alpha"
dirs = ["docs"]

[link_graph.projects.beta]
root = "packages/beta"
dirs = ["docs"]
"#,
    );

    let result = graph_neighbors(fixture.state.as_ref(), "beta/docs/index.md", "both", 1, 10).await;
    let Ok(response) = result else {
        panic!("expected aliased graph neighbors request to succeed");
    };

    assert_studio_json_snapshot(
        "graph_neighbors_vfs_alias_payload",
        json!({
            "center": {
                "id": response.center.id,
                "label": response.center.label,
                "path": response.center.path,
                "nodeType": response.center.node_type,
                "isCenter": response.center.is_center,
                "distance": response.center.distance,
            },
            "nodes": response.nodes.into_iter().map(|node| {
                json!({
                    "id": node.id,
                    "label": node.label,
                    "path": node.path,
                    "nodeType": node.node_type,
                    "isCenter": node.is_center,
                    "distance": node.distance,
                })
            }).collect::<Vec<_>>(),
            "links": response.links.into_iter().map(|link| {
                json!({
                    "source": link.source,
                    "target": link.target,
                    "direction": link.direction,
                    "distance": link.distance,
                })
            }).collect::<Vec<_>>(),
            "totalNodes": response.total_nodes,
            "totalLinks": response.total_links,
        }),
    );
}

#[tokio::test]
async fn graph_neighbors_resolves_relative_markdown_links_from_index_pages() {
    let fixture = make_graph_fixture(vec![
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
    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["docs"]
"#,
    );

    let result = graph_neighbors(fixture.state.as_ref(), "docs/index.md", "both", 1, 20).await;
    let Ok(response) = result else {
        panic!("expected relative markdown links to resolve into graph edges");
    };

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
    assert!(
        response
            .nodes
            .iter()
            .any(|node| node.path.contains("testing/README.md")),
        "expected testing/README.md to be present in graph neighbors"
    );
    assert!(
        response
            .links
            .iter()
            .any(|link| link.target.contains("testing/README")),
        "expected graph links to point at relative markdown targets"
    );
}
