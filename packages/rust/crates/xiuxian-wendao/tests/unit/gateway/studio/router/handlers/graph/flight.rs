use std::sync::Arc;

use xiuxian_db_store::LanceStringArray;

use crate::gateway::studio::router::handlers::graph::flight::{
    graph_neighbors_response_batch, load_graph_neighbors_flight_response,
};
use crate::gateway::studio::router::handlers::graph::tests::build_fixture;
use crate::gateway::studio::types::{
    GraphLink, GraphNeighborsResponse, GraphNode, StudioNavigationTarget,
};

#[tokio::test]
async fn load_graph_neighbors_flight_response_materializes_node_and_link_rows() {
    let fixture = build_fixture(&[
        ("docs/alpha.md", "# Alpha\n\nSee [[beta]].\n"),
        ("docs/beta.md", "# Beta\n\nBody.\n"),
    ]);

    let response = load_graph_neighbors_flight_response(
        Arc::clone(&fixture.state),
        "kernel/docs/alpha.md",
        "both",
        1,
        20,
    )
    .await
    .unwrap_or_else(|error| panic!("load graph-neighbors Flight response: {error:?}"));

    let row_types = response
        .batch
        .column_by_name("rowType")
        .and_then(|column| column.as_any().downcast_ref::<LanceStringArray>())
        .unwrap_or_else(|| panic!("rowType column should decode as Utf8"));
    let navigation_project_name = response
        .batch
        .column_by_name("navigationProjectName")
        .and_then(|column| column.as_any().downcast_ref::<LanceStringArray>())
        .unwrap_or_else(|| panic!("navigationProjectName column should decode as Utf8"));

    assert!(response.app_metadata.is_empty());
    assert_eq!(row_types.value(0), "node");
    assert!(
        row_types.iter().flatten().any(|value| value == "link"),
        "expected one link row in graph-neighbors Flight batch",
    );
    assert_eq!(navigation_project_name.value(0), "kernel");
}

#[test]
fn graph_neighbors_response_batch_preserves_navigation_target_fields() {
    let response = GraphNeighborsResponse {
        center: GraphNode {
            id: "kernel/docs/index.md".to_string(),
            label: "Index".to_string(),
            path: "kernel/docs/index.md".to_string(),
            navigation_target: Some(StudioNavigationTarget {
                path: "kernel/docs/index.md".to_string(),
                category: "doc".to_string(),
                project_name: Some("kernel".to_string()),
                root_label: Some("project".to_string()),
                line: Some(7),
                line_end: Some(9),
                column: Some(3),
            }),
            node_type: "doc".to_string(),
            is_center: true,
            distance: 0,
        },
        nodes: vec![GraphNode {
            id: "kernel/docs/index.md".to_string(),
            label: "Index".to_string(),
            path: "kernel/docs/index.md".to_string(),
            navigation_target: Some(StudioNavigationTarget {
                path: "kernel/docs/index.md".to_string(),
                category: "doc".to_string(),
                project_name: Some("kernel".to_string()),
                root_label: Some("project".to_string()),
                line: Some(7),
                line_end: Some(9),
                column: Some(3),
            }),
            node_type: "doc".to_string(),
            is_center: true,
            distance: 0,
        }],
        links: vec![GraphLink {
            source: "kernel/docs/index.md".to_string(),
            target: "kernel/docs/child.md".to_string(),
            direction: "outgoing".to_string(),
            distance: 1,
        }],
        total_nodes: 1,
        total_links: 1,
    };

    let batch = graph_neighbors_response_batch(&response)
        .unwrap_or_else(|error| panic!("graph-neighbors Flight batch: {error}"));
    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.num_columns(), 18);
}
