use std::sync::Arc;

use crate::gateway::studio::router::handlers::graph::tests::build_fixture;
use crate::gateway::studio::router::handlers::graph::topology::topology_3d;
use axum::extract::State;

#[tokio::test]
async fn topology_3d_returns_non_empty_global_graph_payload() {
    let fixture = build_fixture(&[
        ("alpha.md", "# Alpha\n\nSee [[beta]].\n"),
        ("beta.md", "# Beta\n\nBody.\n"),
    ]);

    let response = topology_3d(State(Arc::clone(&fixture.state)))
        .await
        .unwrap_or_else(|error| panic!("topology request should succeed: {error:?}"))
        .0;

    assert_eq!(response.nodes.len(), 2);
    assert_eq!(response.links.len(), 1);
    assert!(!response.clusters.is_empty());
    assert!(response.nodes.iter().all(|node| node.position.len() == 3));
}
