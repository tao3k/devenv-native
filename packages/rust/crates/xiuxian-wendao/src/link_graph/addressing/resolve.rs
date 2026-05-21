//! `link_graph::addressing::resolve` owns Wendao link graph addressing resolve behavior.

use crate::link_graph::models::{PageIndexMeta, PageIndexNode};

use super::address::{Address, EnhancedResolvedNode, ResolveMode};
use super::errors::ResolveError;
use super::registry::{IndexedNode, RegistryIndex};
use super::topology::{MatchType, PathEntry, PathMatch, TopologyIndex};

/// Resolve a node using dual indices with enhanced match information.
///
/// This is the preferred resolution function for new code, providing:
/// - O(1) ID lookup via `RegistryIndex`
/// - Fuzzy path discovery via `TopologyIndex`
/// - Detailed match type and similarity scoring
///
/// # Errors
///
/// Returns [`ResolveError::NotFound`] when the requested address cannot be resolved from either
/// the registry index or the topology index for the provided document.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn resolve_with_indices(
    registry: &RegistryIndex,
    topology: &TopologyIndex,
    address: &Address,
    doc_id: &str,
    mode: ResolveMode,
) -> Result<EnhancedResolvedNode, ResolveError> {
    match address {
        Address::Id(id) => resolve_id_with_indices(registry, topology, address, doc_id, id),
        Address::Path(components) => {
            resolve_path_with_indices(topology, address, doc_id, components, mode)
        }
        Address::Hash(hash) => resolve_hash_with_indices(topology, address, doc_id, hash),
        Address::Block {
            section_path,
            block_addr: _,
        } => resolve_block_with_indices(topology, address, doc_id, section_path),
    }
}

fn resolve_id_with_indices(
    registry: &RegistryIndex,
    topology: &TopologyIndex,
    address: &Address,
    doc_id: &str,
    id: &str,
) -> Result<EnhancedResolvedNode, ResolveError> {
    if let Some(indexed) = registry.get(id) {
        return Ok(build_registry_resolution(
            indexed,
            Some(id.to_string()),
            MatchType::Exact,
            1.0,
        ));
    }

    topology
        .find_by_hash(id)
        .map(|entry| build_entry_resolution(entry, MatchType::HashFallback, 0.8))
        .ok_or_else(|| not_found(address, doc_id))
}

fn resolve_path_with_indices(
    topology: &TopologyIndex,
    address: &Address,
    doc_id: &str,
    components: &[String],
    mode: ResolveMode,
) -> Result<EnhancedResolvedNode, ResolveError> {
    match mode {
        ResolveMode::Anchor => topology
            .exact_path(doc_id, components)
            .map(|entry| build_entry_resolution(entry, MatchType::Exact, 1.0))
            .or_else(|| {
                topology
                    .path_case_insensitive(doc_id, components)
                    .map(build_path_match_resolution)
            })
            .ok_or_else(|| not_found(address, doc_id)),
        ResolveMode::Discover {
            fuzzy: true,
            max_results,
        } => topology
            .fuzzy_resolve(&components.join("/"), max_results)
            .into_iter()
            .next()
            .map(build_path_match_resolution)
            .ok_or_else(|| not_found(address, doc_id)),
        ResolveMode::Discover { fuzzy: false, .. } => topology
            .exact_path(doc_id, components)
            .map(|entry| build_entry_resolution(entry, MatchType::Exact, 1.0))
            .ok_or_else(|| not_found(address, doc_id)),
    }
}

fn resolve_hash_with_indices(
    topology: &TopologyIndex,
    address: &Address,
    doc_id: &str,
    hash: &str,
) -> Result<EnhancedResolvedNode, ResolveError> {
    topology
        .find_by_hash(hash)
        .map(|entry| build_entry_resolution(entry, MatchType::HashFallback, 0.9))
        .ok_or_else(|| not_found(address, doc_id))
}

fn resolve_block_with_indices(
    topology: &TopologyIndex,
    address: &Address,
    doc_id: &str,
    section_path: &[String],
) -> Result<EnhancedResolvedNode, ResolveError> {
    topology
        .exact_path(doc_id, section_path)
        .map(|entry| build_entry_resolution(entry, MatchType::Exact, 1.0))
        .ok_or_else(|| not_found(address, doc_id))
}

fn build_registry_resolution(
    indexed: &IndexedNode,
    resolved_id: Option<String>,
    match_type: MatchType,
    similarity: f32,
) -> EnhancedResolvedNode {
    EnhancedResolvedNode {
        node: indexed.node.clone(),
        doc_id: indexed.doc_id.clone(),
        resolved_path: indexed.node.metadata.structural_path.clone(),
        resolved_id,
        match_type,
        similarity,
    }
}

fn build_entry_resolution(
    entry: &PathEntry,
    match_type: MatchType,
    similarity: f32,
) -> EnhancedResolvedNode {
    EnhancedResolvedNode {
        node: build_node_from_entry(entry),
        doc_id: entry.doc_id.clone(),
        resolved_path: entry.path.clone(),
        resolved_id: Some(entry.node_id.clone()),
        match_type,
        similarity,
    }
}

fn build_path_match_resolution(path_match: PathMatch) -> EnhancedResolvedNode {
    EnhancedResolvedNode {
        node: build_node_from_entry(&path_match.entry),
        doc_id: path_match.doc_id,
        resolved_path: path_match.path,
        resolved_id: Some(path_match.entry.node_id),
        match_type: path_match.match_type,
        similarity: path_match.similarity_score,
    }
}

fn not_found(address: &Address, doc_id: &str) -> ResolveError {
    ResolveError::NotFound {
        address: address.to_display_string(),
        doc_id: doc_id.to_string(),
    }
}

/// Build a `PageIndexNode` from a `PathEntry`.
///
/// This creates a minimal node with the essential information from the entry.
/// For full node data, use the tree-based resolution.
fn build_node_from_entry(entry: &PathEntry) -> PageIndexNode {
    use std::sync::Arc;

    PageIndexNode {
        node_id: entry.node_id.clone(),
        parent_id: None,
        title: entry.title.clone(),
        level: entry.level,
        text: Arc::from(""),
        summary: None,
        children: Vec::new(),
        blocks: Vec::new(),
        metadata: PageIndexMeta {
            line_range: (0, 0),
            byte_range: None,
            structural_path: entry.path.clone(),
            content_hash: entry.content_hash.clone(),
            attributes: std::collections::HashMap::new(),
            token_count: 0,
            is_thinned: false,
            logbook: Vec::new(),
            observations: Vec::new(),
        },
    }
}
