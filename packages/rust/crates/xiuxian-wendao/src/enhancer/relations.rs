//! `enhancer::relations` owns Wendao enhancer relations behavior.

use crate::link_graph_refs::LinkGraphEntityRef;
use crate::parsers::markdown::{ExplicitSectionRelation, parse_property_relations};

use super::types::InferredRelation;

/// Infer structural relations from note links.
///
/// Ordinary wiki links define graph topology. They do not become typed
/// semantic relations unless another explicit metadata owner adds a relation tag.
#[must_use]
pub fn infer_relations(
    _note_path: &str,
    note_title: &str,
    note_content: &str,
    entity_refs: &[LinkGraphEntityRef],
) -> Vec<InferredRelation> {
    let mut relations = Vec::new();
    let source_note = note_title;

    for entity_ref in entity_refs {
        let target_display = match &entity_ref.target_address {
            Some(address) => format!("{}{}", entity_ref.name, address),
            None => entity_ref.name.clone(),
        };
        relations.push(InferredRelation {
            source: source_note.to_string(),
            source_address: None,
            target: entity_ref.name.clone(),
            target_address: entity_ref.target_address.clone(),
            relation_type: None,
            metadata_owner: None,
            description: format!("{source_note} links to {target_display}"),
        });
    }

    relations.extend(
        parse_property_relations(note_content)
            .into_iter()
            .map(|relation| explicit_relation(note_title, relation)),
    );

    relations
}

fn explicit_relation(note_title: &str, relation: ExplicitSectionRelation) -> InferredRelation {
    let source_address = relation.source.scope_display();
    let target_address = relation
        .target
        .address
        .as_ref()
        .map(crate::link_graph::addressing::Address::to_display_string);
    let target = relation
        .target
        .note_target
        .clone()
        .unwrap_or_else(|| note_title.to_string());
    let target_display = relation.target.display();
    let description = if let Some(source_scope) = &source_address {
        format!(
            "{note_title} at {source_scope} {} {target_display}",
            relation.relation_type
        )
    } else {
        format!("{note_title} {} {target_display}", relation.relation_type)
    };

    InferredRelation {
        source: note_title.to_string(),
        source_address,
        target,
        target_address,
        relation_type: Some(relation.relation_type.to_string()),
        metadata_owner: Some(relation.property_key),
        description,
    }
}
