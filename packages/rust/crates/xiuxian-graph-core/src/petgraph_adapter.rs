//! Optional `petgraph` adapter for reusable graph projections.

use std::collections::BTreeMap;

use petgraph::stable_graph::{NodeIndex, StableDiGraph};

use crate::{GraphNodeId, GraphProjection, GraphProjectionError};

/// Stable directed graph representation produced from a graph projection.
pub type StableGraphProjection = StableDiGraph<String, Option<String>>;

/// Convert a graph projection into a `petgraph` stable directed graph.
///
/// # Errors
///
/// Returns an error when the projection has duplicate node ids or an edge
/// references a missing node.
pub fn to_stable_di_graph(
    projection: &GraphProjection,
) -> Result<StableGraphProjection, GraphProjectionError> {
    projection.validate()?;

    let mut graph = StableDiGraph::<String, Option<String>>::new();
    let mut indexes = BTreeMap::<GraphNodeId, NodeIndex>::new();

    for node in projection.nodes() {
        let index = graph.add_node(node.label().to_string());
        indexes.insert(node.id().clone(), index);
    }

    for edge in projection.edges() {
        let source = indexes.get(edge.source()).copied().ok_or_else(|| {
            GraphProjectionError::MissingSourceNode {
                source_id: edge.source().clone(),
                target_id: edge.target().clone(),
            }
        })?;
        let target = indexes.get(edge.target()).copied().ok_or_else(|| {
            GraphProjectionError::MissingTargetNode {
                source_id: edge.source().clone(),
                target_id: edge.target().clone(),
            }
        })?;
        graph.add_edge(source, target, edge.label().map(str::to_owned));
    }

    Ok(graph)
}
