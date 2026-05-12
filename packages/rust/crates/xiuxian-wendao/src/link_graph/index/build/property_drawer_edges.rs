//! `link_graph::index::build::property_drawer_edges` owns Wendao index build property drawer edges behavior.

use std::collections::HashMap;

use crate::link_graph::addressing::Address;
use crate::link_graph::models::LinkGraphEdgeType;
use crate::parsers::markdown::{ParsedSection, extract_property_relations, normalize_alias};

/// A property drawer edge extracted from attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyDrawerEdge {
    /// Source node ID (`doc_id#anchor` or just `doc_id`).
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Edge type (always `PropertyDrawer`).
    pub edge_type: LinkGraphEdgeType,
    /// The attribute key that defined this edge (e.g., "RELATED").
    pub attribute_key: String,
}

/// Extract property drawer edges for one parsed section.
///
/// This is a graph adapter over the canonical markdown relation parser. It
/// only resolves targets that can be mapped safely with the current graph
/// inputs: document aliases and explicit `:ID:` anchors.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn extract_property_drawer_edges(
    doc_id: &str,
    section: &ParsedSection,
    alias_to_doc_id: &HashMap<String, String>,
) -> Vec<PropertyDrawerEdge> {
    let source_node_id = resolve_source_node_id(doc_id, section);

    extract_property_relations(std::slice::from_ref(section))
        .into_iter()
        .filter_map(|relation| {
            let target_node_id = resolve_target_node_id(doc_id, alias_to_doc_id, &relation.target)?;
            if target_node_id == source_node_id {
                return None;
            }

            Some(PropertyDrawerEdge {
                from: source_node_id.clone(),
                to: target_node_id,
                edge_type: LinkGraphEdgeType::PropertyDrawer,
                attribute_key: relation.property_key,
            })
        })
        .collect()
}

fn resolve_source_node_id(doc_id: &str, section: &ParsedSection) -> String {
    if let Some(explicit_id) = section.attributes.get("ID") {
        return format!("{doc_id}#{explicit_id}");
    }

    if section.heading_path.is_empty() {
        return doc_id.to_string();
    }

    format!("{}#{}", doc_id, section.heading_path.replace(" / ", "/"))
}

fn resolve_target_node_id(
    doc_id: &str,
    alias_to_doc_id: &HashMap<String, String>,
    target: &crate::parsers::markdown::ExplicitRelationTarget,
) -> Option<String> {
    let resolved_doc_id = target
        .note_target
        .as_ref()
        .map(|note_target| normalize_alias(note_target))
        .and_then(|normalized| alias_to_doc_id.get(&normalized).cloned());

    match (&resolved_doc_id, &target.address) {
        (Some(resolved_doc_id), None) => Some(resolved_doc_id.clone()),
        (Some(resolved_doc_id), Some(Address::Id(id))) => Some(format!("{resolved_doc_id}#{id}")),
        (None, Some(Address::Id(id))) => Some(format!("{doc_id}#{id}")),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/link_graph/index/build/property_drawer_edges.rs"]
mod tests;
