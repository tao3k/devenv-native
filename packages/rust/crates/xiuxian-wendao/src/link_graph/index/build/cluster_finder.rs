//! Dense cluster identification for knowledge distillation.
//!
//! Finds subgraphs where nodes have:
//! 1. High saliency (>= `SALIENCY_THRESHOLD_HIGH`)
//! 2. Strong mutual linking (edge density >= `MIN_EDGE_DENSITY`)
//!
//! ## Algorithm
//!
//! Uses a greedy expansion approach:
//! 1. Start from highest-saliency seed nodes
//! 2. Expand to neighbors if they maintain density threshold
//! 3. Stop when no more qualifying neighbors exist
//!
//! ## Usage
//!
//! ```ignore
//! use crate::link_graph::index::build::cluster_finder::{find_dense_clusters, DenseCluster};
//!
//! let clusters = find_dense_clusters(
//!     &high_saliency_nodes,
//!     &outgoing,
//!     &incoming,
//!     &saliency_map,
//! );
//! ```

use super::saliency_snapshot::SALIENCY_THRESHOLD_HIGH;
use std::collections::{HashMap, HashSet};

/// Minimum cluster size (nodes).
pub const MIN_CLUSTER_SIZE: usize = 3;

/// Maximum cluster size (prevents over-expansion).
pub const MAX_CLUSTER_SIZE: usize = 15;

/// Minimum internal edge density for cluster validity.
pub const MIN_EDGE_DENSITY: f64 = 0.4;

/// A dense cluster of high-saliency nodes.
#[derive(Debug, Clone)]
pub struct DenseCluster {
    /// Node IDs in the cluster.
    pub members: Vec<String>,
    /// Average saliency of members.
    pub avg_saliency: f64,
    /// Internal edge count (edges between members).
    pub internal_edges: usize,
    /// Edge density within cluster.
    pub edge_density: f64,
}

impl DenseCluster {
    /// Create a new cluster with the given members.
    #[must_use]
    pub fn new(
        members: Vec<String>,
        saliency_map: &HashMap<String, f64>,
        outgoing: &HashMap<String, HashSet<String>>,
    ) -> Self {
        let avg_saliency = if members.is_empty() {
            0.0
        } else {
            members
                .iter()
                .filter_map(|id| saliency_map.get(id))
                .sum::<f64>()
                / usize_to_f64_saturating(members.len())
        };

        // Count internal edges
        let member_set: HashSet<&String> = members.iter().collect();
        let internal_edges = members
            .iter()
            .filter_map(|member| outgoing.get(member))
            .map(|neighbors| {
                neighbors
                    .iter()
                    .filter(|neighbor| member_set.contains(*neighbor))
                    .count()
            })
            .sum::<usize>();

        // Edge density = actual_edges / possible_edges
        // possible_edges = n * (n-1) for directed graph
        let n = members.len();
        let possible_edges = if n > 1 { n * (n - 1) } else { 1 };
        let edge_density =
            usize_to_f64_saturating(internal_edges) / usize_to_f64_saturating(possible_edges);

        Self {
            members,
            avg_saliency,
            internal_edges,
            edge_density,
        }
    }

    /// Check if cluster meets validity criteria.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.members.len() >= MIN_CLUSTER_SIZE
            && self.edge_density >= MIN_EDGE_DENSITY
            && self.avg_saliency >= SALIENCY_THRESHOLD_HIGH
    }
}

/// Find dense clusters in the graph using greedy expansion.
///
/// # Arguments
/// * `high_saliency_nodes` - Nodes that exceed the saliency threshold
/// * `outgoing` - Map from `node_id` to its outgoing edge targets
/// * `incoming` - Map from `node_id` to its incoming edge sources
/// * `saliency_map` - Map from `node_id` to its saliency value
///
/// # Returns
/// List of valid dense clusters, sorted by average saliency (descending).
#[must_use]
pub fn find_dense_clusters(
    high_saliency_nodes: &[String],
    outgoing: &HashMap<String, HashSet<String>>,
    incoming: &HashMap<String, HashSet<String>>,
    saliency_map: &HashMap<String, f64>,
) -> Vec<DenseCluster> {
    if high_saliency_nodes.len() < MIN_CLUSTER_SIZE {
        return Vec::new();
    }

    let high_set = high_saliency_nodes.iter().collect::<HashSet<_>>();
    let sorted_seeds = sorted_cluster_seeds(high_saliency_nodes, saliency_map);
    let clusters = expand_valid_clusters(sorted_seeds, &high_set, outgoing, incoming, saliency_map);
    sort_clusters_by_saliency(clusters)
}

fn sorted_cluster_seeds<'a>(
    high_saliency_nodes: &'a [String],
    saliency_map: &HashMap<String, f64>,
) -> Vec<&'a String> {
    let mut sorted_seeds = high_saliency_nodes.iter().collect::<Vec<_>>();
    sorted_seeds.sort_by(|a, b| compare_saliency_desc(a, b, saliency_map));
    sorted_seeds
}

fn compare_saliency_desc(
    left: &str,
    right: &str,
    saliency_map: &HashMap<String, f64>,
) -> std::cmp::Ordering {
    let left_score = saliency_map.get(left).unwrap_or(&0.0);
    let right_score = saliency_map.get(right).unwrap_or(&0.0);
    right_score
        .partial_cmp(left_score)
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn expand_valid_clusters(
    sorted_seeds: Vec<&String>,
    high_set: &HashSet<&String>,
    outgoing: &HashMap<String, HashSet<String>>,
    incoming: &HashMap<String, HashSet<String>>,
    saliency_map: &HashMap<String, f64>,
) -> Vec<DenseCluster> {
    let mut visited = HashSet::<String>::new();
    let mut clusters = Vec::<DenseCluster>::new();

    for seed in sorted_seeds {
        if visited.contains(seed) {
            continue;
        }

        let cluster = expand_cluster(seed, &high_set, &visited, outgoing, incoming, saliency_map);

        if cluster.members.len() >= MIN_CLUSTER_SIZE {
            // Mark all members as visited
            for member in &cluster.members {
                visited.insert(member.clone());
            }

            if cluster.is_valid() {
                clusters.push(cluster);
            }
        }
    }

    clusters
}

fn sort_clusters_by_saliency(mut clusters: Vec<DenseCluster>) -> Vec<DenseCluster> {
    clusters.sort_by(|a, b| {
        b.avg_saliency
            .partial_cmp(&a.avg_saliency)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    clusters
}

/// Expand a cluster from a seed node using greedy density optimization.
fn expand_cluster(
    seed: &str,
    high_set: &HashSet<&String>,
    visited: &HashSet<String>,
    outgoing: &HashMap<String, HashSet<String>>,
    incoming: &HashMap<String, HashSet<String>>,
    saliency_map: &HashMap<String, f64>,
) -> DenseCluster {
    let mut members: HashSet<String> = HashSet::new();
    members.insert(seed.to_string());

    // Greedy expansion: add candidate that maximizes density
    while members.len() < MAX_CLUSTER_SIZE {
        let candidates = cluster_candidates(&members, high_set, visited, outgoing, incoming);
        if candidates.is_empty() {
            break;
        }

        // Find best candidate (maintains highest density)
        let best_candidate = candidates
            .iter()
            .map(|candidate| {
                let mut test_members = members.clone();
                test_members.insert(candidate.clone());
                (
                    candidate.clone(),
                    compute_edge_density(&test_members, outgoing),
                )
            })
            .max_by(|(_, left_density), (_, right_density)| {
                left_density
                    .partial_cmp(right_density)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        // Add best candidate if it maintains minimum density
        if let Some((candidate, best_density)) = best_candidate {
            if best_density >= MIN_EDGE_DENSITY {
                members.insert(candidate);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    DenseCluster::new(members.into_iter().collect(), saliency_map, outgoing)
}

fn cluster_candidates(
    members: &HashSet<String>,
    high_set: &HashSet<&String>,
    visited: &HashSet<String>,
    outgoing: &HashMap<String, HashSet<String>>,
    incoming: &HashMap<String, HashSet<String>>,
) -> Vec<String> {
    members
        .iter()
        .flat_map(|member| {
            outgoing
                .get(member)
                .into_iter()
                .chain(incoming.get(member))
                .flat_map(|neighbors| neighbors.iter())
        })
        .filter(|neighbor| {
            high_set.contains(neighbor)
                && !visited.contains(*neighbor)
                && !members.contains(*neighbor)
        })
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

/// Compute edge density within a member set.
fn compute_edge_density(
    members: &HashSet<String>,
    outgoing: &HashMap<String, HashSet<String>>,
) -> f64 {
    if members.len() < 2 {
        return 1.0;
    }

    let internal_edges = members
        .iter()
        .filter_map(|member| outgoing.get(member))
        .map(|neighbors| neighbors.intersection(members).count())
        .sum::<usize>();

    let n = members.len();
    let possible_edges = n * (n - 1);
    usize_to_f64_saturating(internal_edges) / usize_to_f64_saturating(possible_edges)
}

fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}

#[cfg(test)]
#[path = "../../../../tests/unit/link_graph/index/build/cluster_finder.rs"]
mod tests;
