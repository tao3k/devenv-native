//! Unit tests for `skeleton_rerank` module.

use super::{SkeletonRerankOptions, parse_anchor_id, skeleton_rerank};
use crate::link_graph::addressing::{RegistryIndex, TopologyIndex};
use crate::link_graph::models::QuantumAnchorHit;
use crate::link_graph::{PageIndexMeta, PageIndexNode};
use std::collections::HashMap;
use std::sync::Arc;

fn make_test_node(id: &str, title: &str, path: &[&str]) -> PageIndexNode {
    let mut attrs = HashMap::new();
    if !id.is_empty() {
        attrs.insert("ID".to_string(), id.to_string());
    }
    PageIndexNode {
        node_id: format!("doc#{title}").into(),
        parent_id: None,
        title: title.to_string(),
        level: path.len(),
        text: Arc::from("content"),
        summary: None,
        children: Vec::new(),
        blocks: Vec::new(),
        metadata: PageIndexMeta {
            line_range: (1, 10),
            byte_range: Some((0, 100)),
            structural_path: path.iter().map(std::string::ToString::to_string).collect(),
            content_hash: Some(format!("hash_{title}")),
            attributes: attrs,
            token_count: 10,
            is_thinned: false,
            logbook: Vec::new(),
            observations: Vec::new(),
        },
    }
}

fn build_test_indices() -> (RegistryIndex, TopologyIndex) {
    let mut trees = HashMap::new();

    let intro = make_test_node("intro-id", "Introduction", &["Introduction"]);
    let storage = make_test_node("storage-id", "Storage", &["Architecture", "Storage"]);

    trees.insert("test_doc.md".to_string(), vec![intro, storage]);

    let registry = RegistryIndex::build_from_trees(&trees);
    let topology = TopologyIndex::build_from_trees(&trees);

    (registry, topology)
}

#[test]
fn test_skeleton_rerank_validates_existing_anchors() {
    let (registry, topology) = build_test_indices();

    let hits = vec![
        QuantumAnchorHit {
            anchor_id: "test_doc.md#intro-id".to_string(),
            vector_score: 0.8,
        },
        QuantumAnchorHit {
            anchor_id: "test_doc.md#storage-id".to_string(),
            vector_score: 0.7,
        },
    ];

    let results = skeleton_rerank(
        hits,
        &registry,
        &topology,
        &SkeletonRerankOptions::default(),
    );

    assert_eq!(results.len(), 2);
    assert!(results[0].is_valid);
    assert!(results[1].is_valid);
    // First hit should have boosted score
    assert!(results[0].reranked_score > 0.8);
}

#[test]
fn test_skeleton_rerank_penalizes_missing_anchors() {
    let (registry, topology) = build_test_indices();

    let hits = vec![
        QuantumAnchorHit {
            anchor_id: "test_doc.md#intro-id".to_string(),
            vector_score: 0.8,
        },
        QuantumAnchorHit {
            anchor_id: "test_doc.md#deleted-anchor".to_string(),
            vector_score: 0.75,
        },
    ];

    let results = skeleton_rerank(
        hits,
        &registry,
        &topology,
        &SkeletonRerankOptions::default(),
    );

    assert_eq!(results.len(), 2);
    assert!(results[0].is_valid);
    assert!(!results[1].is_valid);
    // Invalid hit should have penalized score
    assert!(results[1].reranked_score < 0.75);
}

#[test]
fn test_skeleton_rerank_filters_invalid_when_strict() {
    let (registry, topology) = build_test_indices();

    let hits = vec![
        QuantumAnchorHit {
            anchor_id: "test_doc.md#intro-id".to_string(),
            vector_score: 0.8,
        },
        QuantumAnchorHit {
            anchor_id: "test_doc.md#deleted-anchor".to_string(),
            vector_score: 0.75,
        },
    ];

    let results = skeleton_rerank(hits, &registry, &topology, &SkeletonRerankOptions::strict());

    assert_eq!(results.len(), 1);
    assert!(results[0].is_valid);
}

#[test]
fn test_parse_anchor_id() {
    assert_eq!(
        parse_anchor_id("doc.md#intro"),
        ("doc.md".to_string(), "intro".to_string())
    );
    assert_eq!(
        parse_anchor_id("path/to/doc.md#section-1"),
        ("path/to/doc.md".to_string(), "section-1".to_string())
    );
    assert_eq!(
        parse_anchor_id("no-hash"),
        ("no-hash".to_string(), String::new())
    );
}

#[test]
fn test_skeleton_rerank_preserves_structural_path() {
    let (registry, topology) = build_test_indices();

    let hits = vec![QuantumAnchorHit {
        anchor_id: "test_doc.md#storage-id".to_string(),
        vector_score: 0.8,
    }];

    let results = skeleton_rerank(
        hits,
        &registry,
        &topology,
        &SkeletonRerankOptions::default(),
    );

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].structural_path,
        Some(vec!["Architecture".to_string(), "Storage".to_string()])
    );
}
