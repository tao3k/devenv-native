//! Unit tests for the shared graph-core crate.

use xiuxian_graph_core::{GraphEdge, GraphNode, GraphProjection, GraphProjectionError};

#[test]
fn graph_projection_reports_missing_edge_target() {
    let projection = GraphProjection::from_parts(
        vec![GraphNode::new("T", "task:demo")],
        vec![GraphEdge::new("T", "N0")],
    );

    assert_eq!(
        projection.validate(),
        Err(GraphProjectionError::MissingTargetNode {
            source_id: "T".to_string(),
            target_id: "N0".to_string(),
        })
    );
}

#[cfg(feature = "mermaid")]
#[test]
fn compact_mermaid_graph_renders_token_efficient_relation_graph() {
    use xiuxian_graph_core::CompactMermaidGraph;

    let projection = GraphProjection::from_parts(
        vec![
            GraphNode::new("T", "task:audio-gate"),
            GraphNode::new("N0", "sdd:.cache/agent/sdd/audio.org"),
            GraphNode::new("N1", "package:xiuxian-wendao-analyzer"),
        ],
        vec![GraphEdge::new("T", "N0"), GraphEdge::new("T", "N1")],
    );

    let output = CompactMermaidGraph::new()
        .render(&projection)
        .unwrap_or_else(|error| panic!("render compact mermaid graph: {error}"));

    assert_eq!(
        output,
        "flowchart LR;T[\"task:audio-gate\"]-->N0[\"sdd:.cache/agent/sdd/audio.org\"];T-->N1[\"package:xiuxian-wendao-analyzer\"]"
    );
}

#[cfg(feature = "petgraph")]
#[test]
fn petgraph_adapter_preserves_node_and_edge_counts() {
    use xiuxian_graph_core::to_stable_di_graph;

    let projection = GraphProjection::from_parts(
        vec![GraphNode::new("A", "source"), GraphNode::new("B", "target")],
        vec![GraphEdge::new("A", "B").with_label("relates")],
    );

    let graph = to_stable_di_graph(&projection)
        .unwrap_or_else(|error| panic!("convert graph projection to petgraph: {error}"));

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);
}
