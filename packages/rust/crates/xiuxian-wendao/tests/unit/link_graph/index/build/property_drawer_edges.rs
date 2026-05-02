//! Unit tests for `property_drawer_edges` module.

use std::collections::HashMap;

use crate::parsers::markdown::ParsedSection;

use super::extract_property_drawer_edges;
use crate::link_graph::models::LinkGraphEdgeType;

fn parsed_section(attrs: &[(&str, &str)], heading_path: &str) -> ParsedSection {
    ParsedSection {
        heading_title: heading_path
            .rsplit(" / ")
            .next()
            .unwrap_or_default()
            .to_string(),
        heading_path: heading_path.to_string(),
        heading_path_lower: heading_path.to_lowercase(),
        heading_level: 2,
        line_start: 1,
        line_end: 3,
        byte_start: 0,
        byte_end: 0,
        section_text: String::new(),
        section_text_lower: String::new(),
        entities: Vec::new(),
        attributes: attrs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
        logbook: Vec::new(),
        observations: Vec::new(),
    }
}

#[test]
fn test_extract_property_drawer_edges_resolve_document_and_id_targets() {
    let section = parsed_section(
        &[
            ("ID", "heading-1"),
            (
                "RELATED",
                "[[file-b#section-2]], [[#local-target]], [[file-b]]",
            ),
        ],
        "Heading 1",
    );
    let alias_to_doc_id = HashMap::from([(String::from("file-b"), String::from("docs/file-b.md"))]);

    let edges = extract_property_drawer_edges("docs/a.md", &section, &alias_to_doc_id);

    assert_eq!(edges.len(), 3);
    assert_eq!(edges[0].from, "docs/a.md#heading-1");
    assert_eq!(edges[0].to, "docs/file-b.md#section-2");
    assert_eq!(edges[0].edge_type, LinkGraphEdgeType::PropertyDrawer);
    assert_eq!(edges[0].attribute_key, "RELATED");
    assert_eq!(edges[1].to, "docs/a.md#local-target");
    assert_eq!(edges[2].to, "docs/file-b.md");
}

#[test]
fn test_extract_property_drawer_edges_skip_unresolved_path_targets() {
    let section = parsed_section(
        &[("RELATED", "[[file-b#/Deep/Section]], [[/Local Path]]")],
        "Heading 1",
    );
    let alias_to_doc_id = HashMap::from([(String::from("file-b"), String::from("docs/file-b.md"))]);

    let edges = extract_property_drawer_edges("docs/a.md", &section, &alias_to_doc_id);

    assert!(edges.is_empty());
}
