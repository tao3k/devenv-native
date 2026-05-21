//! `parsers::markdown::relations::api` owns Wendao markdown relations api behavior.

use super::keys::{PROPERTY_RELATION_KEYS, map_property_relation_type};
use super::targets::parse_relation_targets;
use super::types::{ExplicitRelationSource, ExplicitSectionRelation};
use crate::parsers::markdown::ParsedSection;
use crate::parsers::markdown::content::parse_frontmatter as parse_raw_frontmatter;
use std::collections::HashMap;
use xiuxian_wendao_parsers::sections::{MarkdownSection, extract_sections};

/// Parse explicit property-drawer relations from one markdown document.
///
/// YAML frontmatter is stripped before section parsing. Only explicit
/// property-drawer relation fields participate in this contract.
#[must_use]
pub fn parse_property_relations(content: &str) -> Vec<ExplicitSectionRelation> {
    let (_frontmatter, body) = parse_raw_frontmatter(content);
    let sections = extract_sections(body);
    extract_property_relations_from_markdown_sections(&sections)
}

/// Extract explicit property-drawer relations from parsed markdown sections.
///
/// Each returned row preserves the owning section scope, the explicit property
/// key, and the parsed target note plus optional scoped address.
trait RelationSectionView {
    fn heading_path(&self) -> &str;
    fn attributes(&self) -> &HashMap<String, String>;
}

impl RelationSectionView for ParsedSection {
    fn heading_path(&self) -> &str {
        &self.heading_path
    }

    fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }
}

impl RelationSectionView for MarkdownSection {
    fn heading_path(&self) -> &str {
        self.heading_path()
    }

    fn attributes(&self) -> &HashMap<String, String> {
        self.attributes()
    }
}

fn extract_property_relations_from_view<S: RelationSectionView>(
    sections: &[S],
) -> Vec<ExplicitSectionRelation> {
    let mut relations = Vec::new();

    for section in sections {
        let source = ExplicitRelationSource {
            heading_path: section.heading_path().to_string(),
            explicit_id: section.attributes().get("ID").cloned(),
        };

        for property_key in PROPERTY_RELATION_KEYS {
            let Some(value) = section.attributes().get(*property_key) else {
                continue;
            };
            let Some(relation_type) = map_property_relation_type(property_key) else {
                continue;
            };

            for target in parse_relation_targets(value) {
                relations.push(ExplicitSectionRelation {
                    property_key: (*property_key).to_string(),
                    relation_type: relation_type.clone(),
                    source: source.clone(),
                    target,
                });
            }
        }
    }

    relations
}

/// Extract explicit property-drawer relations from Wendao-enriched sections.
///
/// This compatibility helper remains in `xiuxian-wendao` for existing callers
/// that already hold `ParsedSection` rows. New parser-only flows should prefer
/// parsing raw markdown through [`parse_property_relations`] or consume
/// `xiuxian_wendao_parsers::sections::MarkdownSection` directly.
#[must_use]
pub fn extract_property_relations(sections: &[ParsedSection]) -> Vec<ExplicitSectionRelation> {
    extract_property_relations_from_view(sections)
}

fn extract_property_relations_from_markdown_sections(
    sections: &[MarkdownSection],
) -> Vec<ExplicitSectionRelation> {
    extract_property_relations_from_view(sections)
}
