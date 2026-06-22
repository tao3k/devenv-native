use xiuxian_graph_core::{CompactMermaidGraph, GraphEdge, GraphNode, GraphProjection};

#[test]
fn compact_mermaid_graph_renders_token_efficient_relation_graph() {
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
