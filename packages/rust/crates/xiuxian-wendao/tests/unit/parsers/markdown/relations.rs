//! Unit tests for markdown property relation parsing.

use crate::entity::RelationType;
use crate::link_graph::addressing::Address;

use super::{parse_property_relations, parse_relation_targets};

#[test]
fn test_parse_relation_targets_support_scoped_wiki_links() {
    let targets = parse_relation_targets(
        "[[file-b#section-2]], [[#local-target]], [[/Heading 2]], [[file-c@abcd1234]], @local-hash",
    );

    assert_eq!(targets.len(), 5);

    assert_eq!(targets[0].note_target.as_deref(), Some("file-b"));
    assert_eq!(targets[0].address, Some(Address::id("section-2")));

    assert_eq!(targets[1].note_target, None);
    assert_eq!(targets[1].address, Some(Address::id("local-target")));

    assert_eq!(targets[2].note_target, None);
    assert_eq!(targets[2].address, Some(Address::path(["Heading 2"])));

    assert_eq!(targets[3].note_target.as_deref(), Some("file-c"));
    assert_eq!(targets[3].address, Some(Address::hash("abcd1234")));

    assert_eq!(targets[4].note_target, None);
    assert_eq!(targets[4].address, Some(Address::hash("local-hash")));
}

#[test]
fn test_parse_property_relations_strip_frontmatter_and_ignore_non_link_scalars() {
    let content = r"---
title: Demo
---
## Heading 1
:PROPERTIES:
:ID: heading-1
:RELATED: [[file-b#section-2]], [[#local-target]]
:WEIGHT: 5
:END:
";

    let relations = parse_property_relations(content);

    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].relation_type, RelationType::RelatedTo);
    assert_eq!(relations[0].property_key, "RELATED");
    assert_eq!(
        relations[0].source.explicit_id.as_deref(),
        Some("heading-1")
    );
    assert_eq!(relations[0].target.note_target.as_deref(), Some("file-b"));
    assert_eq!(relations[0].target.address, Some(Address::id("section-2")));

    assert_eq!(relations[1].target.note_target, None);
    assert_eq!(
        relations[1].target.address,
        Some(Address::id("local-target"))
    );
}

#[test]
fn test_parse_property_relations_map_explicit_property_keys() {
    let content = r"
## Architecture
:PROPERTIES:
:DEPENDS_ON: [[foundation#core]]
:EXTENDS: [[base-doc]]
:SEE_ALSO: [[/Appendix]]
:END:
";

    let relations = parse_property_relations(content);

    assert_eq!(relations.len(), 3);
    assert_eq!(relations[0].relation_type, RelationType::DependsOn);
    assert_eq!(relations[1].relation_type, RelationType::Extends);
    assert_eq!(relations[2].relation_type, RelationType::References);
    assert_eq!(
        relations[2].source.scope_display().as_deref(),
        Some("/Architecture")
    );
    assert_eq!(
        relations[2].target.address,
        Some(Address::path(["Appendix"]))
    );
}
