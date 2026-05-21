//! `link_graph::addressing::node_lookup` owns Wendao link graph addressing node lookup behavior.

use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::link_graph::models::{
    BlockAddress, MarkdownBlock, PageIndexNode, markdown_block_matches_kind,
};

use super::address::{Address, ResolvedNode};

/// Resolve a node within a document using the Triple-A protocol.
///
/// Attempts resolution in order: ID → Path → Hash.
/// Returns `None` if no resolution succeeds.
#[must_use]
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn resolve_node<S>(
    trees: &HashMap<String, Vec<PageIndexNode>, S>,
    address: &Address,
    doc_id: &str,
) -> Option<ResolvedNode>
where
    S: BuildHasher,
{
    let nodes = trees.get(doc_id)?;

    match address {
        Address::Id(id) => {
            if let Some(node) = find_by_id(nodes, id) {
                return Some(ResolvedNode {
                    node,
                    doc_id: doc_id.to_string(),
                    migrated_from: None,
                });
            }

            if let Some(node) = find_by_path_from_id(nodes, id) {
                return Some(ResolvedNode {
                    node,
                    doc_id: doc_id.to_string(),
                    migrated_from: Some(address.clone()),
                });
            }

            if let Some(node) = find_by_hash_from_id(nodes, id) {
                return Some(ResolvedNode {
                    node,
                    doc_id: doc_id.to_string(),
                    migrated_from: Some(address.clone()),
                });
            }

            None
        }
        Address::Path(components) => {
            if let Some(node) = find_by_path(nodes, components) {
                return Some(ResolvedNode {
                    node,
                    doc_id: doc_id.to_string(),
                    migrated_from: None,
                });
            }

            if let Some(node) = find_by_hash_from_path(nodes, components) {
                return Some(ResolvedNode {
                    node,
                    doc_id: doc_id.to_string(),
                    migrated_from: Some(address.clone()),
                });
            }

            None
        }
        Address::Hash(hash) => find_by_hash(nodes, hash).map(|node| ResolvedNode {
            node,
            doc_id: doc_id.to_string(),
            migrated_from: None,
        }),
        Address::Block {
            section_path,
            block_addr,
        } => {
            if let Some(node) = find_by_path(nodes, section_path)
                && find_block_in_node(&node, block_addr).is_some()
            {
                return Some(ResolvedNode {
                    node,
                    doc_id: doc_id.to_string(),
                    migrated_from: None,
                });
            }
            None
        }
    }
}

/// Find a node by explicit `:ID:` attribute.
pub(super) fn find_by_id(nodes: &[PageIndexNode], id: &str) -> Option<PageIndexNode> {
    for node in nodes {
        if node
            .metadata
            .attributes
            .get("ID")
            .map(std::string::String::as_str)
            == Some(id)
        {
            return Some(node.clone());
        }
        if let Some(found) = find_by_id(&node.children, id) {
            return Some(found);
        }
    }
    None
}

/// Find a node by structural path.
pub(super) fn find_by_path(
    nodes: &[PageIndexNode],
    components: &[String],
) -> Option<PageIndexNode> {
    if components.is_empty() {
        return None;
    }

    for node in nodes {
        if node.metadata.structural_path.as_slice() == components {
            return Some(node.clone());
        }

        if node.title == components[0] {
            if components.len() == 1 {
                return Some(node.clone());
            }
            if let Some(found) = find_by_path(&node.children, &components[1..]) {
                return Some(found);
            }
        }

        if let Some(found) = find_by_path(&node.children, components) {
            return Some(found);
        }
    }
    None
}

/// Find a node by content hash.
pub(super) fn find_by_hash(nodes: &[PageIndexNode], hash: &str) -> Option<PageIndexNode> {
    for node in nodes {
        if node.metadata.content_hash.as_deref() == Some(hash) {
            return Some(node.clone());
        }
        if let Some(found) = find_by_hash(&node.children, hash) {
            return Some(found);
        }
    }
    None
}

/// Fallback: try to find by path when ID lookup failed.
fn find_by_path_from_id(_nodes: &[PageIndexNode], _id: &str) -> Option<PageIndexNode> {
    None
}

/// Fallback: try to find by hash when ID lookup failed.
fn find_by_hash_from_id(_nodes: &[PageIndexNode], _id: &str) -> Option<PageIndexNode> {
    None
}

/// Fallback: try to find by hash when path lookup failed.
fn find_by_hash_from_path(
    _nodes: &[PageIndexNode],
    _components: &[String],
) -> Option<PageIndexNode> {
    None
}

/// Find a specific block within a node by block address.
fn find_block_in_node<'a>(
    node: &'a PageIndexNode,
    block_addr: &BlockAddress,
) -> Option<&'a MarkdownBlock> {
    let matching_blocks: Vec<_> = node
        .blocks
        .iter()
        .filter(|block| markdown_block_matches_kind(block, &block_addr.kind))
        .collect();

    matching_blocks.get(block_addr.index).copied()
}
