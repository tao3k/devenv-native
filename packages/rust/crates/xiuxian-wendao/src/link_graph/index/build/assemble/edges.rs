use std::collections::{HashMap, HashSet};

use crate::link_graph::index::build::assemble::types::EdgeTables;
use crate::parsers::markdown::{ParsedNote, normalize_alias};

pub(crate) fn build_edge_tables(
    parsed_notes: &[ParsedNote],
    alias_to_doc_id: &HashMap<String, String>,
) -> EdgeTables {
    let (outgoing, incoming, edge_count) = parsed_notes.iter().fold(
        (
            HashMap::<String, HashSet<String>>::new(),
            HashMap::<String, HashSet<String>>::new(),
            0usize,
        ),
        |(mut outgoing, mut incoming, edge_count), parsed| {
            let from_id = &parsed.doc.id;
            let markdown_edge_count = insert_markdown_link_edges(
                from_id,
                &parsed.link_targets,
                alias_to_doc_id,
                &mut outgoing,
                &mut incoming,
            );
            let property_edge_count = insert_property_drawer_edges(
                from_id,
                &parsed.sections,
                alias_to_doc_id,
                &mut outgoing,
                &mut incoming,
            );
            (
                outgoing,
                incoming,
                edge_count + markdown_edge_count + property_edge_count,
            )
        },
    );

    EdgeTables {
        outgoing,
        incoming,
        edge_count,
    }
}

fn insert_markdown_link_edges(
    from_id: &str,
    link_targets: &[String],
    alias_to_doc_id: &HashMap<String, String>,
    outgoing: &mut HashMap<String, HashSet<String>>,
    incoming: &mut HashMap<String, HashSet<String>>,
) -> usize {
    link_targets
        .iter()
        .filter_map(|raw_target| {
            let normalized = normalize_alias(raw_target);
            (!normalized.is_empty())
                .then(|| alias_to_doc_id.get(&normalized).cloned())
                .flatten()
        })
        .filter(|to_id| to_id != from_id)
        .filter(|to_id| insert_edge(from_id, to_id, outgoing, incoming))
        .count()
}

fn insert_property_drawer_edges(
    from_id: &str,
    sections: &[crate::parsers::markdown::ParsedSection],
    alias_to_doc_id: &HashMap<String, String>,
    outgoing: &mut HashMap<String, HashSet<String>>,
    incoming: &mut HashMap<String, HashSet<String>>,
) -> usize {
    sections
        .iter()
        .flat_map(|section| {
            crate::link_graph::index::build::property_drawer_edges::extract_property_drawer_edges(
                from_id,
                section,
                alias_to_doc_id,
            )
        })
        .filter(|edge| edge.to != from_id)
        .filter(|edge| insert_edge(edge.from.as_str(), edge.to.as_str(), outgoing, incoming))
        .count()
}

fn insert_edge(
    from_id: &str,
    to_id: &str,
    outgoing: &mut HashMap<String, HashSet<String>>,
    incoming: &mut HashMap<String, HashSet<String>>,
) -> bool {
    let inserted = outgoing
        .entry(from_id.to_string())
        .or_default()
        .insert(to_id.to_string());
    if inserted {
        incoming
            .entry(to_id.to_string())
            .or_default()
            .insert(from_id.to_string());
    }
    inserted
}
