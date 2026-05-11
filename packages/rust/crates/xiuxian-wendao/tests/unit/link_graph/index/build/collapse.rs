//! Unit tests for graph collapse operators.

use std::collections::{HashMap, HashSet};

use super::{DenseCluster, VirtualNode, collapse_clusters};
use crate::link_graph::models::LinkGraphDocument;

fn make_cluster(members: &[&str], avg_saliency: f64) -> DenseCluster {
    DenseCluster {
        members: members
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
        avg_saliency,
        internal_edges: members.len() * 2,
        edge_density: 0.5,
    }
}

fn make_doc(id: &str, stem: &str) -> LinkGraphDocument {
    LinkGraphDocument {
        id: id.to_string(),
        id_lower: id.to_lowercase(),
        stem: stem.to_string(),
        stem_lower: stem.to_lowercase(),
        path: format!("{id}.md").into(),
        path_lower: format!("{}.md", id.to_lowercase()),
        title: stem.to_string(),
        title_lower: stem.to_lowercase(),
        tags: Vec::new(),
        tags_lower: Vec::new(),
        lead: String::new(),
        doc_type: None,
        word_count: 0,
        search_text: String::new(),
        search_text_lower: String::new(),
        saliency_base: 0.5,
        decay_rate: 0.1,
        created_ts: None,
        modified_ts: None,
    }
}

#[test]
fn test_virtual_node_id_generation() {
    let members = vec!["a.md".to_string(), "b.md".to_string(), "c.md".to_string()];
    let id = VirtualNode::generate_id(&members, 0);
    assert!(id.starts_with("virtual:cluster:0:"));
}

#[test]
fn test_virtual_node_title_synthesis() {
    let titles = vec!["Understanding Performance Optimization"];
    let title = VirtualNode::synthesize_title(&titles);
    assert!(title.contains("Cluster:"));
}

#[test]
fn test_collapse_empty_clusters() {
    let docs_by_id = HashMap::new();
    let mut outgoing = HashMap::new();
    let mut incoming = HashMap::new();

    let result = collapse_clusters(vec![], &docs_by_id, &mut outgoing, &mut incoming);
    assert!(result.is_empty());
}

#[test]
fn test_collapse_single_cluster() {
    let docs_by_id: HashMap<String, LinkGraphDocument> = [
        ("a.md".to_string(), make_doc("a.md", "Doc A")),
        ("b.md".to_string(), make_doc("b.md", "Doc B")),
        ("c.md".to_string(), make_doc("c.md", "Doc C")),
    ]
    .into_iter()
    .collect();
    let mut outgoing: HashMap<String, HashSet<String>> = HashMap::new();
    let mut incoming: HashMap<String, HashSet<String>> = HashMap::new();

    // Setup: a -> d (external), b -> c (internal, c -> e (external)
    outgoing.insert(
        "a.md".to_string(),
        ["d.md".to_string()].into_iter().collect(),
    );
    outgoing.insert(
        "b.md".to_string(),
        ["c.md".to_string()].into_iter().collect(),
    );
    outgoing.insert(
        "c.md".to_string(),
        ["e.md".to_string()].into_iter().collect(),
    );

    // b has incoming from x (external)
    incoming.insert(
        "b.md".to_string(),
        ["x.md".to_string()].into_iter().collect(),
    );

    let cluster = make_cluster(&["a.md", "b.md", "c.md"], 0.85);
    let result = collapse_clusters(vec![cluster], &docs_by_id, &mut outgoing, &mut incoming);

    assert_eq!(result.len(), 1);
    let vn = &result[0];
    assert_eq!(vn.members.len(), 3);
    assert!((vn.avg_saliency - 0.85).abs() < 0.01);

    // Check edge rewiring
    assert!(vn.outgoing_edges.contains("d.md"));
    assert!(vn.outgoing_edges.contains("e.md"));
    assert!(vn.incoming_edges.contains("x.md"));
}
