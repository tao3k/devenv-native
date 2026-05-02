//! Tests for `LinkGraph` entity reference extraction.

use crate::link_graph_refs::{
    LinkGraphEntityRef, LinkGraphRefStats, extract_entity_refs, find_notes_referencing_entity,
    get_ref_stats, is_valid_entity_ref, parse_entity_ref,
};

#[test]
fn test_extract_single_ref() {
    let content = "See [[FactoryPattern]] for details.";
    let refs = extract_entity_refs(content);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].name, "FactoryPattern");
    assert_eq!(refs[0].target_address, None);
}

#[test]
fn test_extract_addressed_ref() {
    let content = "Use [[SingletonPattern#Examples]] implementation.";
    let refs = extract_entity_refs(content);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].name, "SingletonPattern");
    assert_eq!(refs[0].target_address.as_deref(), Some("#Examples"));
}

#[test]
fn test_extract_multiple_refs() {
    let content = "See [[FactoryPattern]], [[SingletonPattern#Examples]], and [[ObserverPattern]].";
    let refs = extract_entity_refs(content);
    assert_eq!(refs.len(), 3);
}

#[test]
fn test_extract_refs_with_alias() {
    let content = "Use [[FactoryPattern|FP]] for creation.";
    let refs = extract_entity_refs(content);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].name, "FactoryPattern");
    assert_eq!(refs[0].original, "[[FactoryPattern|FP]]");
}

#[test]
fn test_deduplicate_refs() {
    let content = "First [[FactoryPattern]] then [[FactoryPattern]] again.";
    let refs = extract_entity_refs(content);
    assert_eq!(refs.len(), 1);
}

#[test]
fn test_empty_content() {
    let content = "";
    let refs = extract_entity_refs(content);
    assert!(refs.is_empty());
}

#[test]
fn test_no_refs() {
    let content = "Just regular text without any links.";
    let refs = extract_entity_refs(content);
    assert!(refs.is_empty());
}

#[test]
fn test_to_wikilink() {
    let ref1 = LinkGraphEntityRef::new(
        "FactoryPattern".to_string(),
        None,
        "[[FactoryPattern]]".to_string(),
    );
    assert_eq!(ref1.to_wikilink(), "[[FactoryPattern]]");

    let ref2 = LinkGraphEntityRef::new(
        "SingletonPattern".to_string(),
        Some("#Examples".to_string()),
        "[[SingletonPattern#Examples]]".to_string(),
    );
    assert_eq!(ref2.to_wikilink(), "[[SingletonPattern#Examples]]");
}

#[test]
fn test_to_tag() {
    let ref1 = LinkGraphEntityRef::new(
        "FactoryPattern".to_string(),
        None,
        "[[FactoryPattern]]".to_string(),
    );
    assert_eq!(ref1.to_tag(), "#entity");

    let ref2 = LinkGraphEntityRef::new(
        "SingletonPattern".to_string(),
        Some("#Examples".to_string()),
        "[[SingletonPattern#Examples]]".to_string(),
    );
    assert_eq!(ref2.to_tag(), "#entity-addressed");
}

#[test]
fn test_count_entity_refs() {
    let content = "[[A]] [[B]] [[C]]";
    assert_eq!(extract_entity_refs(content).len(), 3);
}

#[test]
fn test_is_valid_entity_ref() {
    assert!(is_valid_entity_ref("[[FactoryPattern]]"));
    assert!(is_valid_entity_ref("[[SingletonPattern#Examples]]"));
    assert!(!is_valid_entity_ref("not a ref"));
    assert!(is_valid_entity_ref("[[Pattern|alias]]"));
}

#[test]
fn test_parse_entity_ref() {
    let Some(ref1) = parse_entity_ref("[[FactoryPattern]]") else {
        panic!("expected valid entity reference");
    };
    assert_eq!(ref1.name, "FactoryPattern");
    assert_eq!(ref1.target_address, None);

    let Some(ref2) = parse_entity_ref("[[SingletonPattern#Examples]]") else {
        panic!("expected addressed entity reference");
    };
    assert_eq!(ref2.name, "SingletonPattern");
    assert_eq!(ref2.target_address.as_deref(), Some("#Examples"));
}

#[test]
fn test_extract_entity_refs_skips_local_only_addresses() {
    let refs = extract_entity_refs("See [[#Implementation]] and [[FactoryPattern]].");

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].name, "FactoryPattern");
}

#[test]
fn test_extract_entity_refs_skips_embedded_wikilinks() {
    let refs = extract_entity_refs("![[FactoryPattern]] and [[ObserverPattern]].");

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].name, "ObserverPattern");
}

#[test]
fn test_find_notes_referencing_entity() {
    let notes = vec![
        ("note1", "See [[FactoryPattern]]"),
        ("note2", "SingletonPattern is related"),
        ("note3", "Check [[FactoryPattern#Examples]]"),
        ("note4", "FactoryPattern appears only as plain text"),
    ];

    let refs = find_notes_referencing_entity("FactoryPattern", &notes);
    assert_eq!(refs.len(), 2);
    assert!(refs.contains(&"note1"));
    assert!(refs.contains(&"note3"));
    assert!(!refs.contains(&"note4"));
}

#[test]
fn test_ref_stats() {
    let refs = vec![
        LinkGraphEntityRef::new(
            "A".to_string(),
            Some("#Alpha".to_string()),
            "[[A#Alpha]]".to_string(),
        ),
        LinkGraphEntityRef::new(
            "B".to_string(),
            Some("#Beta".to_string()),
            "[[B#Beta]]".to_string(),
        ),
        LinkGraphEntityRef::new("C".to_string(), None, "[[C]]".to_string()),
    ];

    let stats = LinkGraphRefStats::from_refs(&refs);
    assert_eq!(stats.total_refs, 3);
    assert_eq!(stats.unique_entities, 3);
    assert_eq!(stats.by_type.len(), 2);
}

#[test]
fn test_get_ref_stats() {
    let content = "[[A#Alpha]] [[B#Beta]] [[C]]";
    let stats = get_ref_stats(content);
    assert_eq!(stats.total_refs, 3);
    assert_eq!(stats.unique_entities, 3);
}
