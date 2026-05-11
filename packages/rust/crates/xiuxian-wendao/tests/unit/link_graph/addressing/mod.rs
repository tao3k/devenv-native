//! Unit tests for addressing mod module.

use super::{
    Address, ModificationError, adjust_line_range, replace_byte_range, update_section_content,
};
use crate::link_graph::PageIndexNode;
use std::collections::HashMap;

pub(super) use super::mutation::compute_hash;
pub(super) use super::node_lookup::{find_by_hash, find_by_id, find_by_path};

#[test]
fn test_address_parse_id() {
    let addr = Address::parse("#arch-v1");
    assert_eq!(addr, Some(Address::Id("arch-v1".to_string())));
}

#[test]
fn test_address_parse_path() {
    let addr = Address::parse("/Architecture/Storage");
    assert_eq!(
        addr,
        Some(Address::Path(vec![
            "Architecture".to_string(),
            "Storage".to_string()
        ]))
    );
}

#[test]
fn test_address_parse_hash() {
    let addr = Address::parse("@a1b2c3d4e5f6");
    assert_eq!(addr, Some(Address::Hash("a1b2c3d4e5f6".to_string())));
}

#[test]
fn test_address_parse_empty() {
    assert!(Address::parse("").is_none());
    assert!(Address::parse("#").is_none());
    assert!(Address::parse("@").is_none());
}

#[test]
fn test_address_to_display_string() {
    assert_eq!(Address::id("test").to_display_string(), "#test");
    assert_eq!(Address::path(vec!["A", "B"]).to_display_string(), "/A/B");
    assert_eq!(Address::hash("abc123").to_display_string(), "@abc123");
}

#[test]
fn test_find_by_id() {
    let node = PageIndexNode {
        node_id: "doc#section".to_string().into(),
        parent_id: None,
        title: "Section".to_string(),
        level: 1,
        text: std::sync::Arc::from("content"),
        summary: None,
        children: Vec::new(),
        blocks: Vec::new(),
        metadata: crate::link_graph::models::PageIndexMeta {
            line_range: (1, 10),
            byte_range: Some((0, 100)),
            structural_path: vec!["Section".to_string()],
            content_hash: Some("abc123".to_string()),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("ID".to_string(), "my-section".to_string());
                attrs
            },
            token_count: 10,
            is_thinned: false,
            logbook: Vec::new(),
            observations: Vec::new(),
        },
    };

    let found = find_by_id(std::slice::from_ref(&node), "my-section");
    assert!(found.is_some());

    let not_found = find_by_id(&[node], "other-id");
    assert!(not_found.is_none());
}

#[test]
fn test_find_by_path() {
    let node = PageIndexNode {
        node_id: "doc#storage".to_string().into(),
        parent_id: None,
        title: "Storage".to_string(),
        level: 2,
        text: std::sync::Arc::from("content"),
        summary: None,
        children: Vec::new(),
        blocks: Vec::new(),
        metadata: crate::link_graph::models::PageIndexMeta {
            line_range: (1, 10),
            byte_range: Some((0, 100)),
            structural_path: vec!["Architecture".to_string(), "Storage".to_string()],
            content_hash: None,
            attributes: HashMap::new(),
            token_count: 10,
            is_thinned: false,
            logbook: Vec::new(),
            observations: Vec::new(),
        },
    };

    let found = find_by_path(
        std::slice::from_ref(&node),
        &["Architecture".to_string(), "Storage".to_string()],
    );
    assert!(found.is_some());

    let found_by_title = find_by_path(&[node], &["Storage".to_string()]);
    assert!(found_by_title.is_some());
}

#[test]
fn test_find_by_hash() {
    let node = PageIndexNode {
        node_id: "doc#section".to_string().into(),
        parent_id: None,
        title: "Section".to_string(),
        level: 1,
        text: std::sync::Arc::from("content"),
        summary: None,
        children: Vec::new(),
        blocks: Vec::new(),
        metadata: crate::link_graph::models::PageIndexMeta {
            line_range: (1, 10),
            byte_range: Some((0, 100)),
            structural_path: vec![],
            content_hash: Some("def456".to_string()),
            attributes: HashMap::new(),
            token_count: 10,
            is_thinned: false,
            logbook: Vec::new(),
            observations: Vec::new(),
        },
    };

    let found = find_by_hash(std::slice::from_ref(&node), "def456");
    assert!(found.is_some());

    let not_found = find_by_hash(&[node], "other-hash");
    assert!(not_found.is_none());
}

#[test]
fn test_replace_byte_range_basic() {
    let content = "Hello, world!";
    let Ok(result) = replace_byte_range(content, 7, 12, "Rust", None) else {
        panic!("replace_byte_range should succeed");
    };
    assert_eq!(result.new_content, "Hello, Rust!");
    assert_eq!(result.byte_delta, -1);
    assert_eq!(result.line_delta, 0);
}

#[test]
fn test_replace_byte_range_with_hash_verification() {
    let content = "Hello, world!";
    let hash = compute_hash("world");
    let Ok(result) = replace_byte_range(content, 7, 12, "Rust", Some(&hash)) else {
        panic!("replace_byte_range should verify the hash");
    };
    assert_eq!(result.new_content, "Hello, Rust!");
}

#[test]
fn test_replace_byte_range_hash_mismatch() {
    let content = "Hello, world!";
    let result = replace_byte_range(content, 7, 12, "Rust", Some("wronghash"));
    assert!(matches!(
        result,
        Err(ModificationError::HashMismatch { .. })
    ));
}

#[test]
fn test_replace_byte_range_out_of_bounds() {
    let content = "Hello";
    let result = replace_byte_range(content, 0, 100, "test", None);
    assert!(matches!(
        result,
        Err(ModificationError::ByteRangeOutOfBounds { .. })
    ));
}

#[test]
fn test_update_section_content() {
    let node = PageIndexNode {
        node_id: "doc#section".to_string().into(),
        parent_id: None,
        title: "Section".to_string(),
        level: 1,
        text: std::sync::Arc::from("old content"),
        summary: None,
        children: Vec::new(),
        blocks: Vec::new(),
        metadata: crate::link_graph::models::PageIndexMeta {
            line_range: (1, 5),
            byte_range: Some((0, 11)),
            structural_path: vec![],
            content_hash: Some(compute_hash("old content")),
            attributes: HashMap::new(),
            token_count: 2,
            is_thinned: false,
            logbook: Vec::new(),
            observations: Vec::new(),
        },
    };

    let doc_content = "old content here";
    let Ok(result) = update_section_content(doc_content, &node, "new content") else {
        panic!("update_section_content should succeed");
    };
    assert_eq!(result.new_content, "new content here");
    assert_eq!(result.byte_delta, 0);
}

#[test]
fn test_adjust_line_range_before() {
    let (start, end) = adjust_line_range(10, 20, 5, 5);
    assert_eq!(start, 15);
    assert_eq!(end, 25);
}

#[test]
fn test_adjust_line_range_within() {
    let (start, end) = adjust_line_range(10, 20, 3, 15);
    assert_eq!(start, 10);
    assert_eq!(end, 23);
}

#[test]
fn test_adjust_line_range_after() {
    let (start, end) = adjust_line_range(10, 20, 5, 30);
    assert_eq!(start, 10);
    assert_eq!(end, 20);
}

#[test]
fn test_adjust_line_range_before_negative_delta() {
    let (start, end) = adjust_line_range(10, 20, -3, 5);
    assert_eq!(start, 7);
    assert_eq!(end, 17);
}

#[test]
fn test_compute_hash_consistency() {
    let hash1 = compute_hash("test content");
    let hash2 = compute_hash("test content");
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 16);
}
