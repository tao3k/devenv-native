use super::support::{
    make_graph_fixture, push_ui_config_from_toml, sorted_graph_links_payload,
    sorted_graph_nodes_payload,
};
use super::*;

#[tokio::test]
async fn node_neighbors_returns_live_neighbors() {
    let fixture = make_graph_fixture(vec![
        ("alpha.md", "# Alpha\n\nSee [[beta]].\n"),
        ("beta.md", "# Beta\n\nSee [[gamma]].\n"),
        ("gamma.md", "# Gamma\n\nTail node.\n"),
    ]);
    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["."]
"#,
    );

    let result = node_neighbors(fixture.state.as_ref(), "alpha.md").await;
    let Ok(response) = result else {
        panic!("expected node neighbors request to succeed");
    };

    assert_studio_json_snapshot(
        "graph_node_neighbors",
        json!({
            "nodeId": response.node_id,
            "name": response.name,
            "nodeType": response.node_type,
            "incoming": response.incoming,
            "outgoing": response.outgoing,
            "twoHop": response.two_hop,
        }),
    );
}

#[tokio::test]
async fn graph_neighbors_includes_center_node_and_links() {
    let fixture = make_graph_fixture(vec![
        ("alpha.md", "# Alpha\n\nSee [[beta]].\n"),
        ("beta.md", "# Beta\n\nBody.\n"),
    ]);
    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["."]
"#,
    );

    let result = graph_neighbors(fixture.state.as_ref(), "alpha.md", "both", 2, 10).await;
    let Ok(response) = result else {
        panic!("expected graph neighbors request to succeed");
    };

    assert_studio_json_snapshot(
        "graph_neighbors_payload",
        json!({
            "center": {
                "id": response.center.id,
                "label": response.center.label,
                "path": response.center.path,
                "navigationTarget": response.center.navigation_target,
                "nodeType": response.center.node_type,
                "isCenter": response.center.is_center,
                "distance": response.center.distance,
            },
            "nodes": sorted_graph_nodes_payload(response.nodes),
            "links": sorted_graph_links_payload(response.links),
            "totalNodes": response.total_nodes,
            "totalLinks": response.total_links,
        }),
    );
}

#[tokio::test]
async fn graph_neighbors_returns_not_found_for_unknown_node() {
    let fixture = make_graph_fixture(vec![("alpha.md", "# Alpha\n\nBody.\n")]);
    push_ui_config_from_toml(
        &fixture,
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["."]
"#,
    );

    let result = graph_neighbors(fixture.state.as_ref(), "missing.md", "both", 2, 10).await;
    let Err(error) = result else {
        panic!("expected missing node lookup to fail");
    };

    assert_eq!(error.status(), axum::http::StatusCode::NOT_FOUND);
    assert_eq!(error.code(), "NOT_FOUND");
}
