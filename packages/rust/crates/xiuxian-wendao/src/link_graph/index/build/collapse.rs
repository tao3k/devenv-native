//! Graph collapse operators for knowledge distillation.
//!
//! Collapses dense clusters into `VirtualNodes` that:
//! 1. Inherit all outgoing/incoming edges of member nodes
//! 2. Store references to original member IDs
//! 3. Get synthesized stem/title from cluster essence
//!
//! ## Usage
//!
//! ```ignore
//! use crate::link_graph::index::build::collapse::{collapse_clusters, VirtualNode};
//!
//! let virtual_nodes = collapse_clusters(&clusters, &docs_by_id, &mut outgoing, &mut incoming);
//! ```

use super::cluster_finder::DenseCluster;
use crate::link_graph::models::LinkGraphDocument;
use std::collections::{HashMap, HashSet};
use std::hash::Hasher;

/// A virtual node created from collapsing a dense cluster.
#[derive(Debug, Clone)]
pub struct VirtualNode {
    /// Synthesized identifier for the virtual node.
    pub id: String,
    /// Original member node IDs that were collapsed.
    pub members: Vec<String>,
    /// Average saliency of collapsed nodes.
    pub avg_saliency: f64,
    /// Synthesized title (e.g., "Cluster: essence-topic").
    pub title: String,
    /// Internal edge count (edges between members).
    pub internal_edges: usize,
    /// Edge density within cluster.
    pub edge_density: f64,
    /// All outgoing edges from members to non-members.
    pub outgoing_edges: HashSet<String>,
    /// All incoming edges from non-members to members.
    pub incoming_edges: HashSet<String>,
}

impl VirtualNode {
    /// Generate a virtual node ID from cluster members.
    #[must_use]
    pub fn generate_id(members: &[String], cluster_index: usize) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        for m in members {
            hasher.write(m.as_bytes());
        }
        let hash_val = hasher.finish();
        format!("virtual:cluster:{cluster_index}:{hash_val:08x}")
    }

    /// Generate a title from member titles (first 3 words of top member).
    #[must_use]
    pub fn synthesize_title(member_titles: &[&str]) -> String {
        if member_titles.is_empty() {
            return "Collapsed Cluster".to_string();
        }

        // Take first 3 words from all member titles
        let words: Vec<&str> = member_titles
            .iter()
            .flat_map(|t| t.split_whitespace().take(3))
            .collect();
        format!("Cluster: {}", words.join(" "))
    }
}

/// Collapse dense clusters into virtual nodes.
///
/// # Arguments
/// * `clusters` - Dense clusters to collapse
/// * `docs_by_id` - Document map (read-only, used for title extraction)
/// * `outgoing` - Outgoing edge map (will be modified)
/// * `incoming` - Incoming edge map (will be modified)
///
/// # Returns
/// Vector of `VirtualNodes` created
pub fn collapse_clusters(
    clusters: Vec<DenseCluster>,
    docs_by_id: &HashMap<String, LinkGraphDocument>,
    outgoing: &mut HashMap<String, HashSet<String>>,
    incoming: &mut HashMap<String, HashSet<String>>,
) -> Vec<VirtualNode> {
    if clusters.is_empty() {
        return Vec::new();
    }

    let virtual_nodes = build_virtual_nodes(clusters, docs_by_id, outgoing, incoming);
    rewire_virtual_nodes(&virtual_nodes, outgoing, incoming);
    virtual_nodes
}

fn build_virtual_nodes(
    clusters: Vec<DenseCluster>,
    docs_by_id: &HashMap<String, LinkGraphDocument>,
    outgoing: &HashMap<String, HashSet<String>>,
    incoming: &HashMap<String, HashSet<String>>,
) -> Vec<VirtualNode> {
    clusters
        .into_iter()
        .enumerate()
        .map(|(cluster_index, cluster)| {
            build_virtual_node(cluster_index, cluster, docs_by_id, outgoing, incoming)
        })
        .collect()
}

fn build_virtual_node(
    cluster_index: usize,
    cluster: DenseCluster,
    docs_by_id: &HashMap<String, LinkGraphDocument>,
    outgoing: &HashMap<String, HashSet<String>>,
    incoming: &HashMap<String, HashSet<String>>,
) -> VirtualNode {
    let (outgoing_edges, incoming_edges) = cluster_external_edges(&cluster, outgoing, incoming);
    let member_titles = member_titles(&cluster.members, docs_by_id);
    VirtualNode {
        id: VirtualNode::generate_id(&cluster.members, cluster_index),
        members: cluster.members,
        avg_saliency: cluster.avg_saliency,
        title: VirtualNode::synthesize_title(&member_titles),
        internal_edges: cluster.internal_edges,
        edge_density: cluster.edge_density,
        outgoing_edges,
        incoming_edges,
    }
}

fn cluster_external_edges(
    cluster: &DenseCluster,
    outgoing: &HashMap<String, HashSet<String>>,
    incoming: &HashMap<String, HashSet<String>>,
) -> (HashSet<String>, HashSet<String>) {
    let member_set = cluster.members.iter().collect::<HashSet<_>>();
    (
        external_edges(&cluster.members, &member_set, outgoing),
        external_edges(&cluster.members, &member_set, incoming),
    )
}

fn external_edges(
    members: &[String],
    member_set: &HashSet<&String>,
    edge_map: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    members
        .iter()
        .filter_map(|member_id| edge_map.get(member_id))
        .flat_map(|neighbors| neighbors.iter())
        .filter(|neighbor| !member_set.contains(*neighbor))
        .cloned()
        .collect()
}

fn member_titles<'a>(
    members: &[String],
    docs_by_id: &'a HashMap<String, LinkGraphDocument>,
) -> Vec<&'a str> {
    members
        .iter()
        .filter_map(|id| docs_by_id.get(id).map(|doc| doc.stem.as_str()))
        .collect()
}

fn rewire_virtual_nodes(
    virtual_nodes: &[VirtualNode],
    outgoing: &mut HashMap<String, HashSet<String>>,
    incoming: &mut HashMap<String, HashSet<String>>,
) {
    for virtual_node in virtual_nodes {
        remove_internal_member_edges(virtual_node, outgoing, incoming);
        insert_virtual_node_edges(virtual_node, outgoing, incoming);
        insert_reverse_virtual_node_edges(virtual_node, outgoing, incoming);
    }
}

fn remove_internal_member_edges(
    virtual_node: &VirtualNode,
    outgoing: &mut HashMap<String, HashSet<String>>,
    incoming: &mut HashMap<String, HashSet<String>>,
) {
    let member_set = virtual_node.members.iter().collect::<HashSet<_>>();
    for member_id in &virtual_node.members {
        if let Some(neighbors) = outgoing.get_mut(member_id) {
            neighbors.retain(|neighbor| !member_set.contains(neighbor));
        }
        if let Some(neighbors) = incoming.get_mut(member_id) {
            neighbors.retain(|neighbor| !member_set.contains(neighbor));
        }
    }
}

fn insert_virtual_node_edges(
    virtual_node: &VirtualNode,
    outgoing: &mut HashMap<String, HashSet<String>>,
    incoming: &mut HashMap<String, HashSet<String>>,
) {
    outgoing
        .entry(virtual_node.id.clone())
        .or_default()
        .extend(virtual_node.outgoing_edges.iter().cloned());
    incoming
        .entry(virtual_node.id.clone())
        .or_default()
        .extend(virtual_node.incoming_edges.iter().cloned());
}

fn insert_reverse_virtual_node_edges(
    virtual_node: &VirtualNode,
    outgoing: &mut HashMap<String, HashSet<String>>,
    incoming: &mut HashMap<String, HashSet<String>>,
) {
    for ext_node in &virtual_node.outgoing_edges {
        incoming
            .entry(ext_node.clone())
            .or_default()
            .insert(virtual_node.id.clone());
    }

    for ext_node in &virtual_node.incoming_edges {
        outgoing
            .entry(ext_node.clone())
            .or_default()
            .insert(virtual_node.id.clone());
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/link_graph/index/build/collapse.rs"]
mod tests;
