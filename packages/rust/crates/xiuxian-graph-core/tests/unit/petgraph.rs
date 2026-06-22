use xiuxian_graph_core::{GraphEdge, GraphNode, GraphProjection, to_stable_di_graph};

#[test]
fn petgraph_adapter_preserves_node_and_edge_counts() {
    let projection = GraphProjection::from_parts(
        vec![GraphNode::new("A", "source"), GraphNode::new("B", "target")],
        vec![GraphEdge::new("A", "B").with_label("relates")],
    );

    let graph = to_stable_di_graph(&projection)
        .unwrap_or_else(|error| panic!("convert graph projection to petgraph: {error}"));

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);
}
