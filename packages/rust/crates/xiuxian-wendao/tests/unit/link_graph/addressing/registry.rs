//! Unit tests for registry module.

use super::RegistryIndex;
use crate::link_graph::PageIndexNode;
use std::collections::HashMap;
use std::sync::Arc;

fn make_test_node(id: &str, title: &str) -> PageIndexNode {
    let mut attrs = HashMap::new();
    if !id.is_empty() {
        attrs.insert("ID".to_string(), id.to_string());
    }
    PageIndexNode {
        node_id: format!("doc#{title}"),
        parent_id: None,
        title: title.to_string(),
        level: 1,
        text: Arc::from("content"),
        summary: None,
        children: Vec::new(),
        blocks: Vec::new(),
        metadata: crate::link_graph::PageIndexMeta {
            line_range: (1, 10),
            byte_range: Some((0, 100)),
            structural_path: vec![title.to_string()],
            content_hash: Some("abc123".to_string()),
            attributes: attrs,
            token_count: 10,
            is_thinned: false,
            logbook: Vec::new(),
            observations: Vec::new(),
        },
    }
}

#[test]
fn test_empty_registry() {
    let registry = RegistryIndex::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(!registry.contains("any-id"));
}

#[test]
fn test_build_from_trees() {
    let mut trees = HashMap::new();
    trees.insert(
        "doc1.md".to_string(),
        vec![
            make_test_node("intro", "Introduction"),
            make_test_node("arch", "Architecture"),
        ],
    );
    trees.insert(
        "doc2.md".to_string(),
        vec![make_test_node("config", "Configuration")],
    );

    let registry = RegistryIndex::build_from_trees(&trees);

    assert_eq!(registry.len(), 3);
    assert!(registry.contains("intro"));
    assert!(registry.contains("arch"));
    assert!(registry.contains("config"));
}

#[test]
fn test_get_returns_correct_doc() {
    let mut trees = HashMap::new();
    trees.insert(
        "doc1.md".to_string(),
        vec![make_test_node("my-id", "Section")],
    );

    let registry = RegistryIndex::build_from_trees(&trees);

    let Some(indexed) = registry.get("my-id") else {
        panic!("registry should contain my-id");
    };
    assert_eq!(indexed.doc_id, "doc1.md");
    assert_eq!(indexed.node.title, "Section");
}

#[test]
fn test_nested_nodes_indexed() {
    let child = make_test_node("child-id", "Child Section");
    let parent = PageIndexNode {
        node_id: "doc#parent".to_string(),
        parent_id: None,
        title: "Parent".to_string(),
        level: 1,
        text: Arc::from("parent content"),
        summary: None,
        children: vec![child],
        blocks: Vec::new(),
        metadata: crate::link_graph::PageIndexMeta {
            line_range: (1, 20),
            byte_range: Some((0, 200)),
            structural_path: vec!["Parent".to_string()],
            content_hash: None,
            attributes: HashMap::new(),
            token_count: 20,
            is_thinned: false,
            logbook: Vec::new(),
            observations: Vec::new(),
        },
    };

    let mut trees = HashMap::new();
    trees.insert("doc.md".to_string(), vec![parent]);

    let registry = RegistryIndex::build_from_trees(&trees);

    assert_eq!(registry.len(), 1);
    assert!(registry.contains("child-id"));
}

#[test]
fn test_nodes_without_id_not_indexed() {
    let mut trees = HashMap::new();
    trees.insert(
        "doc.md".to_string(),
        vec![
            make_test_node("", "NoID1"), // Empty ID = not indexed
            make_test_node("has-id", "HasID"),
            make_test_node("", "NoID2"), // Empty ID = not indexed
        ],
    );

    let registry = RegistryIndex::build_from_trees(&trees);

    assert_eq!(registry.len(), 1);
    assert!(registry.contains("has-id"));
    assert!(!registry.contains("")); // Empty ID not indexed
}

#[test]
fn test_doc_ids() {
    let mut trees = HashMap::new();
    trees.insert(
        "doc1.md".to_string(),
        vec![make_test_node("id1", "Section 1")],
    );
    trees.insert(
        "doc2.md".to_string(),
        vec![make_test_node("id2", "Section 2")],
    );

    let registry = RegistryIndex::build_from_trees(&trees);
    let docs = registry.doc_ids();

    assert_eq!(docs.len(), 2);
}

#[test]
fn test_collision_detection_no_collisions() {
    let mut trees = HashMap::new();
    trees.insert(
        "doc1.md".to_string(),
        vec![make_test_node("intro", "Introduction")],
    );
    trees.insert(
        "doc2.md".to_string(),
        vec![make_test_node("arch", "Architecture")],
    );

    let result = RegistryIndex::build_from_trees_with_collisions(&trees);

    assert!(result.collisions.is_empty());
    assert_eq!(result.registry.len(), 2);
}

#[test]
fn test_collision_detection_with_collisions() {
    let mut trees = HashMap::new();
    // Same ID "intro" in two different documents
    trees.insert(
        "doc1.md".to_string(),
        vec![make_test_node("intro", "Introduction 1")],
    );
    trees.insert(
        "doc2.md".to_string(),
        vec![make_test_node("intro", "Introduction 2")],
    );

    let result = RegistryIndex::build_from_trees_with_collisions(&trees);

    assert_eq!(result.collisions.len(), 1);
    assert_eq!(result.collisions[0].id, "intro");
    assert_eq!(result.collisions[0].locations.len(), 2);

    // Registry still works (last occurrence wins)
    assert!(result.registry.contains("intro"));
}

#[test]
fn test_collision_detection_multiple_collisions() {
    let mut trees = HashMap::new();
    // Multiple IDs duplicated
    trees.insert(
        "doc1.md".to_string(),
        vec![
            make_test_node("shared-id", "Section A"),
            make_test_node("another-dup", "Section B"),
        ],
    );
    trees.insert(
        "doc2.md".to_string(),
        vec![
            make_test_node("shared-id", "Section C"),
            make_test_node("another-dup", "Section D"),
        ],
    );
    trees.insert(
        "doc3.md".to_string(),
        vec![make_test_node("unique-id", "Section E")],
    );

    let result = RegistryIndex::build_from_trees_with_collisions(&trees);

    assert_eq!(result.collisions.len(), 2);
    let collision_ids: Vec<&str> = result.collisions.iter().map(|c| c.id.as_str()).collect();
    assert!(collision_ids.contains(&"shared-id"));
    assert!(collision_ids.contains(&"another-dup"));
}
