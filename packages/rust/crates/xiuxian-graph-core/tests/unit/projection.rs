use xiuxian_graph_core::{
    GraphEdge, GraphNode, GraphNodeId, GraphProjection, GraphProjectionError,
};

#[test]
fn graph_projection_reports_missing_edge_target() {
    let projection = GraphProjection::from_parts(
        vec![GraphNode::new("T", "task:demo")],
        vec![GraphEdge::new("T", "N0")],
    );

    assert_eq!(
        projection.validate(),
        Err(GraphProjectionError::MissingTargetNode {
            source_id: GraphNodeId::from("T"),
            target_id: GraphNodeId::from("N0"),
        })
    );
}
