use super::support::{make_graph_fixture, push_ui_config_from_toml, round_f32};
use super::*;

#[tokio::test]
async fn topology_3d_returns_nodes_and_links() {
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

    let result = topology_3d(fixture.state.as_ref()).await;
    let Ok(response) = result else {
        panic!("expected topology request to succeed");
    };

    let mut nodes = response
        .nodes
        .into_iter()
        .map(|node| {
            json!({
                "id": node.id,
                "name": node.name,
                "nodeType": node.node_type,
                "position": node.position.map(round_f32),
                "clusterId": node.cluster_id,
            })
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));

    let mut links = response
        .links
        .into_iter()
        .map(|link| {
            json!({
                "from": link.from,
                "to": link.to,
                "label": link.label,
            })
        })
        .collect::<Vec<_>>();
    links.sort_by(|left, right| {
        left["from"]
            .as_str()
            .cmp(&right["from"].as_str())
            .then_with(|| left["to"].as_str().cmp(&right["to"].as_str()))
    });

    let mut clusters = response
        .clusters
        .into_iter()
        .map(|cluster| {
            json!({
                "id": cluster.id,
                "name": cluster.name,
                "centroid": cluster.centroid.map(round_f32),
                "nodeCount": cluster.node_count,
                "color": cluster.color,
            })
        })
        .collect::<Vec<_>>();
    clusters.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));

    assert_studio_json_snapshot(
        "topology_3d_payload",
        json!({
            "nodes": nodes,
            "links": links,
            "clusters": clusters,
        }),
    );
}
