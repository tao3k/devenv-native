use super::support::{make_graph_fixture, push_ui_config_from_toml};
use super::*;

#[tokio::test]
async fn graph_neighbors_indexes_configured_projects_outside_knowledge_root() {
    let fixture = make_graph_fixture(vec![
        ("docs/overview.md", "# Overview\n\nKernel docs.\n"),
        (
            ".data/wendao-frontend/docs/03_features/202_topology_and_graph_navigation.md",
            "# Topology\n\nSee [[overview]].\n",
        ),
    ]);
    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["docs"]

[link_graph.projects.main]
root = ".data/wendao-frontend"
dirs = ["docs"]
"#,
    );

    let result = graph_neighbors(
        fixture.state.as_ref(),
        "main/docs/03_features/202_topology_and_graph_navigation.md",
        "both",
        1,
        10,
    )
    .await;
    let Ok(response) = result else {
        panic!("expected configured project graph neighbors request to succeed");
    };

    assert_studio_json_snapshot(
        "graph_configured_project_alias_payload",
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
async fn graph_neighbors_rebuilds_after_ui_config_update() {
    let fixture = make_graph_fixture(vec![
        ("docs/overview.md", "# Overview\n\nKernel docs.\n"),
        (
            ".data/wendao-frontend/docs/03_features/202_topology_and_graph_navigation.md",
            "# Topology\n\nSee [[overview]].\n",
        ),
    ]);

    let missing = graph_neighbors(
        fixture.state.as_ref(),
        "main/docs/03_features/202_topology_and_graph_navigation.md",
        "both",
        1,
        10,
    )
    .await;
    let Err(error) = missing else {
        panic!("expected graph request to fail before ui config is pushed");
    };

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);

    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["docs"]

[link_graph.projects.main]
root = ".data/wendao-frontend"
dirs = ["docs"]
"#,
    );

    let rebuilt = graph_neighbors(
        fixture.state.as_ref(),
        "main/docs/03_features/202_topology_and_graph_navigation.md",
        "both",
        1,
        10,
    )
    .await;
    let Ok(response) = rebuilt else {
        panic!("expected graph request to succeed after ui config update");
    };

    assert_eq!(
        response.center.path,
        "main/docs/03_features/202_topology_and_graph_navigation.md"
    );
}
