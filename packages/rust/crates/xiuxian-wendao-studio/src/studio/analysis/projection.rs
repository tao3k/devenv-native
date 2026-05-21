use std::fmt::Write;

use crate::studio::types::{
    AnalysisEdge, AnalysisEdgeKind, AnalysisNode, AnalysisNodeKind, MermaidProjection,
    MermaidViewKind,
};

pub(crate) fn build_mermaid_projections(
    nodes: &[AnalysisNode],
    edges: &[AnalysisEdge],
) -> Vec<MermaidProjection> {
    vec![
        build_outline_projection(nodes, edges),
        build_task_projection(nodes, edges),
    ]
}

fn build_outline_projection(nodes: &[AnalysisNode], edges: &[AnalysisEdge]) -> MermaidProjection {
    let mut source = String::from("graph TD\n");
    let node_count = nodes
        .iter()
        .filter(|node| matches!(node.kind, AnalysisNodeKind::Section))
        .inspect(|node| {
            let _ = writeln!(source, "  {}[\"{}\"]", escape_id(&node.id), node.label);
        })
        .count();

    let edge_count = edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.kind,
                AnalysisEdgeKind::Contains | AnalysisEdgeKind::Parent
            )
        })
        .inspect(|edge| {
            let s_id = escape_id(&edge.source_id);
            let t_id = escape_id(&edge.target_id);
            let _ = writeln!(source, "  {s_id} --> {t_id}");
        })
        .count();

    MermaidProjection {
        kind: MermaidViewKind::Outline,
        source,
        node_count,
        edge_count,
    }
}

fn build_task_projection(nodes: &[AnalysisNode], edges: &[AnalysisEdge]) -> MermaidProjection {
    let mut source = String::from("graph LR\n");
    let node_count = nodes
        .iter()
        .filter(|node| matches!(node.kind, AnalysisNodeKind::Task))
        .inspect(|node| {
            let _ = writeln!(source, "  {}[\"{}\"]", escape_id(&node.id), node.label);
        })
        .count();

    let edge_count = edges
        .iter()
        .filter(|edge| matches!(edge.kind, AnalysisEdgeKind::NextStep))
        .inspect(|edge| {
            let _ = writeln!(
                source,
                "  {} --> {}",
                escape_id(&edge.source_id),
                escape_id(&edge.target_id)
            );
        })
        .count();

    MermaidProjection {
        kind: MermaidViewKind::Tasks,
        source,
        node_count,
        edge_count,
    }
}

fn escape_id(id: &str) -> String {
    id.replace([':', '.', '-'], "_")
}
