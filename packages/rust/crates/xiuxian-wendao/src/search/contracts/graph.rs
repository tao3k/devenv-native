use serde::{Deserialize, Serialize};
use specta::Type;

use super::StudioNavigationTarget;

/// A single node in the link-graph visualization.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    /// Global identifier for the node.
    pub id: String,
    /// Display label.
    pub label: String,
    /// File path if the node represents a document.
    pub path: String,
    /// Display-ready navigation target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation_target: Option<StudioNavigationTarget>,
    /// Optional node type (e.g., "CORE", "FEATURE").
    pub node_type: String,
    /// Whether this is the focal node of the query.
    pub is_center: bool,
    /// Shortest-path distance from the center node.
    pub distance: usize,
}

/// A single edge in the link-graph visualization.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GraphLink {
    /// Source node identifier.
    pub source: String,
    /// Target node identifier.
    pub target: String,
    /// Relationship direction label.
    pub direction: String,
    /// Hop distance for this edge.
    pub distance: usize,
}

/// Result of a graph neighbor traversal.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GraphNeighborsResponse {
    /// Center node.
    pub center: GraphNode,
    /// Nodes in the neighbor subgraph.
    pub nodes: Vec<GraphNode>,
    /// Links connecting the neighbors.
    pub links: Vec<GraphLink>,
    /// Number of returned nodes.
    pub total_nodes: usize,
    /// Number of returned links.
    pub total_links: usize,
}

/// Payload for 3D graph topology visualization.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Topology3dPayload {
    /// All nodes in the global graph.
    pub nodes: Vec<TopologyNode>,
    /// All edges in the global graph.
    pub links: Vec<TopologyLink>,
    /// Cluster summaries for grouped graph rendering.
    pub clusters: Vec<TopologyCluster>,
}

/// A single node in the 3D topology graph.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TopologyNode {
    /// Global identifier for the node.
    pub id: String,
    /// Human-friendly node label.
    pub name: String,
    /// Node category.
    pub node_type: String,
    /// Initial 3D position.
    pub position: [f32; 3],
    /// Optional cluster identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_id: Option<String>,
}

/// A single edge in the 3D topology graph.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TopologyLink {
    /// Source node identifier.
    pub from: String,
    /// Target node identifier.
    pub to: String,
    /// Optional edge label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Cluster metadata for the 3D topology graph.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TopologyCluster {
    /// Cluster identifier.
    pub id: String,
    /// Human-friendly cluster label.
    pub name: String,
    /// Cluster centroid.
    pub centroid: [f32; 3],
    /// Number of nodes in the cluster.
    pub node_count: usize,
    /// Stable cluster display color.
    pub color: String,
}
