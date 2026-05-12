//! Query-core-native graph projections built from Arrow relations.

use std::collections::{BTreeSet, HashSet};

use arrow::array::{Array, StringArray, UInt64Array};

use crate::link_graph::{LinkGraphDirection, LinkGraphIndex};
use crate::query_core::types::{WendaoQueryCoreError, WendaoRelation};

/// Query-core-native graph node projection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct WendaoGraphNode {
    /// Stable graph node id from the link-graph index.
    pub node_id: String,
    /// Internal repository-relative path.
    pub path: String,
    /// Best-effort title from graph metadata.
    pub title: String,
    /// Shortest-path distance from the center node.
    pub distance: usize,
    /// Whether this row is the center node of the traversal.
    pub is_center: bool,
}

/// Query-core-native graph link projection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WendaoGraphLink {
    /// Internal source path.
    pub source_path: String,
    /// Internal target path.
    pub target_path: String,
    /// Direction label.
    pub direction: String,
    /// Hop distance for the edge.
    pub distance: usize,
}

/// Query-core-native projection of a graph-neighbor traversal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WendaoGraphProjection {
    /// Center node for the traversal.
    pub center: WendaoGraphNode,
    /// Unique projected nodes, including the center node.
    pub nodes: Vec<WendaoGraphNode>,
    /// Unique projected links among the included nodes.
    pub links: Vec<WendaoGraphLink>,
}

impl WendaoGraphProjection {
    /// Collect unique internal paths from the projection.
    #[must_use]
    pub fn paths_at_distance(&self, exact_distance: Option<usize>) -> Vec<String> {
        let mut paths = self
            .nodes
            .iter()
            .filter(|node| !node.is_center)
            .filter(|node| exact_distance.is_none_or(|expected| node.distance == expected))
            .map(|node| node.path.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        paths.sort();
        paths
    }
}

/// Build a query-core-native graph projection from a graph-neighbor relation.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the relation schema is invalid or the
/// center node cannot be determined.
pub fn graph_projection_from_relation(
    index: &LinkGraphIndex,
    relation: &WendaoRelation,
) -> Result<WendaoGraphProjection, WendaoQueryCoreError> {
    let mut nodes = Vec::<WendaoGraphNode>::new();
    let mut seen_paths = HashSet::<String>::new();

    for batch in relation.batches() {
        let node_ids = batch
            .column_by_name("node_id")
            .and_then(|array| array.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                WendaoQueryCoreError::InvalidRelation(
                    "graph neighbor relation is missing `node_id`".to_string(),
                )
            })?;
        let paths = batch
            .column_by_name("path")
            .and_then(|array| array.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                WendaoQueryCoreError::InvalidRelation(
                    "graph neighbor relation is missing `path`".to_string(),
                )
            })?;
        let titles = batch
            .column_by_name("title")
            .and_then(|array| array.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                WendaoQueryCoreError::InvalidRelation(
                    "graph neighbor relation is missing `title`".to_string(),
                )
            })?;
        let distances = batch
            .column_by_name("distance")
            .and_then(|array| array.as_any().downcast_ref::<UInt64Array>())
            .ok_or_else(|| {
                WendaoQueryCoreError::InvalidRelation(
                    "graph neighbor relation is missing `distance`".to_string(),
                )
            })?;

        for row in 0..batch.num_rows() {
            let path = paths.value(row).to_string();
            if !seen_paths.insert(path.clone()) {
                continue;
            }

            let distance = usize::try_from(distances.value(row)).unwrap_or(usize::MAX);
            nodes.push(WendaoGraphNode {
                node_id: node_ids.value(row).to_string(),
                path,
                title: if titles.is_null(row) {
                    String::new()
                } else {
                    titles.value(row).to_string()
                },
                distance,
                is_center: distance == 0,
            });
        }
    }

    nodes.sort_by(|left, right| {
        right
            .is_center
            .cmp(&left.is_center)
            .then_with(|| left.distance.cmp(&right.distance))
            .then_with(|| left.path.cmp(&right.path))
    });

    let center = nodes
        .iter()
        .find(|node| node.is_center)
        .cloned()
        .ok_or_else(|| {
            WendaoQueryCoreError::InvalidRelation(
                "graph neighbor relation is missing center row".to_string(),
            )
        })?;
    let links = collect_projection_links(index, nodes.as_slice());
    Ok(WendaoGraphProjection {
        center,
        nodes,
        links,
    })
}

fn collect_projection_links(
    index: &LinkGraphIndex,
    nodes: &[WendaoGraphNode],
) -> Vec<WendaoGraphLink> {
    let included_paths = nodes
        .iter()
        .map(|node| node.path.clone())
        .collect::<HashSet<_>>();
    let mut links = Vec::<WendaoGraphLink>::new();
    let mut seen_links = HashSet::<(String, String)>::new();

    for source in nodes {
        let edge_limit = index
            .neighbor_count(source.path.as_str(), LinkGraphDirection::Outgoing)
            .max(1);
        for outgoing in index.neighbors(
            source.path.as_str(),
            LinkGraphDirection::Outgoing,
            1,
            edge_limit,
        ) {
            if source.path == outgoing.path || !included_paths.contains(outgoing.path.as_str()) {
                continue;
            }
            let key = (source.path.clone(), outgoing.path.clone());
            if seen_links.insert(key.clone()) {
                links.push(WendaoGraphLink {
                    source_path: key.0,
                    target_path: key.1,
                    direction: "outgoing".to_string(),
                    distance: 1,
                });
            }
        }
    }

    links.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then_with(|| left.target_path.cmp(&right.target_path))
            .then_with(|| left.direction.cmp(&right.direction))
    });
    links
}
