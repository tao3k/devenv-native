//! Unit tests for `cluster_finder` module.

use std::collections::{HashMap, HashSet};

use super::{DenseCluster, MIN_CLUSTER_SIZE, MIN_EDGE_DENSITY, find_dense_clusters};

fn make_saliency_map(nodes: &[(&str, f64)]) -> HashMap<String, f64> {
    nodes.iter().map(|(id, s)| (id.to_string(), *s)).collect()
}

fn make_edge_map(edges: &[(&str, &str)]) -> HashMap<String, HashSet<String>> {
    let mut map: HashMap<String, HashSet<String>> = HashMap::new();
    for (from, to) in edges {
        map.entry(from.to_string())
            .or_default()
            .insert(to.to_string());
    }
    map
}

#[test]
fn test_empty_cluster() {
    let saliency = make_saliency_map(&[]);
    let outgoing = HashMap::new();
    let cluster = DenseCluster::new(vec![], &saliency, &outgoing);
    assert_eq!(cluster.members.len(), 0);
    assert!(cluster.avg_saliency.abs() < f64::EPSILON);
    assert!(!cluster.is_valid());
}

#[test]
fn test_cluster_validity() {
    let saliency = make_saliency_map(&[("a", 0.8), ("b", 0.85), ("c", 0.9)]);
    // a -> b, b -> c, c -> a (density = 3/6 = 0.5)
    let outgoing = make_edge_map(&[("a", "b"), ("b", "c"), ("c", "a")]);
    let cluster = DenseCluster::new(
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        &saliency,
        &outgoing,
    );
    assert!(cluster.is_valid());
    assert!((cluster.edge_density - 0.5).abs() < 0.01);
}

#[test]
fn test_low_density_cluster_invalid() {
    let saliency = make_saliency_map(&[("a", 0.8), ("b", 0.85), ("c", 0.9)]);
    // Only a -> b (density = 1/6 = 0.167)
    let outgoing = make_edge_map(&[("a", "b")]);
    let cluster = DenseCluster::new(
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        &saliency,
        &outgoing,
    );
    assert!(!cluster.is_valid()); // density < MIN_EDGE_DENSITY
}

#[test]
fn test_find_clusters_insufficient_nodes() {
    let high = vec!["a".to_string(), "b".to_string()];
    let outgoing = HashMap::new();
    let incoming = HashMap::new();
    let saliency = make_saliency_map(&[("a", 0.8), ("b", 0.85)]);

    let clusters = find_dense_clusters(&high, &outgoing, &incoming, &saliency);
    assert!(clusters.is_empty());
}

#[test]
fn test_min_cluster_size_constant() {
    assert_eq!(MIN_CLUSTER_SIZE, 3);
}

#[test]
fn test_min_edge_density_constant() {
    assert!((MIN_EDGE_DENSITY - 0.4).abs() < f64::EPSILON);
}
