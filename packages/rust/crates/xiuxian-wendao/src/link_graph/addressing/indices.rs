//! `link_graph::addressing::indices` owns Wendao link graph addressing indices behavior.

use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::link_graph::models::PageIndexNode;

/// Build a reverse index from ID to (`doc_id`, node).
#[must_use]
pub fn build_id_index<S>(
    trees: &HashMap<String, Vec<PageIndexNode>, S>,
) -> HashMap<String, (String, String)>
where
    S: BuildHasher,
{
    let mut index = HashMap::new();
    for (doc_id, nodes) in trees {
        collect_ids(nodes, doc_id, &mut index);
    }
    index
}

fn collect_ids(
    nodes: &[PageIndexNode],
    doc_id: &str,
    index: &mut HashMap<String, (String, String)>,
) {
    for node in nodes {
        if let Some(id) = node.metadata.attributes.get("ID") {
            index.insert(id.clone(), (doc_id.to_string(), node.node_id.clone()));
        }
        collect_ids(&node.children, doc_id, index);
    }
}

/// Build a reverse index from content hash to (`doc_id`, `node_id`).
#[must_use]
pub fn build_hash_index<S>(
    trees: &HashMap<String, Vec<PageIndexNode>, S>,
) -> HashMap<String, (String, String)>
where
    S: BuildHasher,
{
    let mut index = HashMap::new();
    for (doc_id, nodes) in trees {
        collect_hashes(nodes, doc_id, &mut index);
    }
    index
}

fn collect_hashes(
    nodes: &[PageIndexNode],
    doc_id: &str,
    index: &mut HashMap<String, (String, String)>,
) {
    for node in nodes {
        if let Some(hash) = &node.metadata.content_hash {
            index.insert(hash.clone(), (doc_id.to_string(), node.node_id.clone()));
        }
        collect_hashes(&node.children, doc_id, index);
    }
}
