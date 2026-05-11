use super::{LinkGraphIndex, SymbolCacheStats, SymbolRef};
use crate::link_graph::PageIndexMeta;
use crate::link_graph::models::{LinkGraphDocument, PageIndexNode};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_symbol_cache_stats_empty() {
    let stats = SymbolCacheStats {
        unique_symbols: 0,
        total_references: 0,
    };
    assert_eq!(stats.unique_symbols, 0);
    assert_eq!(stats.total_references, 0);
}

#[test]
fn test_symbol_cache_stats_with_data() {
    let stats = SymbolCacheStats {
        unique_symbols: 10,
        total_references: 25,
    };
    assert_eq!(stats.unique_symbols, 10);
    assert_eq!(stats.total_references, 25);
}

#[test]
fn test_symbol_ref_serialization() {
    let symbol_ref = SymbolRef {
        doc_id: "docs/api".to_string().into(),
        node_id: "docs/api#section-1".to_string().into(),
        pattern: "fn process_data($$$)".to_string(),
        language: "rust".to_string(),
        line_number: Some(42),
        scope: Some("src/api/**".to_string()),
    };

    let Ok(json) = serde_json::to_string(&symbol_ref) else {
        panic!("symbol reference serialization should succeed");
    };
    assert!(json.contains("process_data"));
    assert!(json.contains("rust"));
    assert!(json.contains("src/api"));

    let Ok(deserialized) = serde_json::from_str::<SymbolRef>(&json) else {
        panic!("symbol reference deserialization should succeed");
    };
    assert_eq!(deserialized.doc_id, "docs/api");
    assert_eq!(deserialized.line_number, Some(42));
    assert_eq!(deserialized.scope, Some("src/api/**".to_string()));
}

#[test]
fn test_symbol_ref_serialization_no_scope() {
    let symbol_ref = SymbolRef {
        doc_id: "docs/api".to_string().into(),
        node_id: "docs/api#section-1".to_string().into(),
        pattern: "fn process_data($$$)".to_string(),
        language: "rust".to_string(),
        line_number: Some(42),
        scope: None,
    };

    let Ok(json) = serde_json::to_string(&symbol_ref) else {
        panic!("symbol reference serialization should succeed");
    };
    let Ok(deserialized) = serde_json::from_str::<SymbolRef>(&json) else {
        panic!("symbol reference deserialization should succeed");
    };
    assert!(deserialized.scope.is_none());
}

#[test]
fn test_page_index_lineage_helpers_follow_canonical_owner_surface() {
    let doc_id = "notes/a".to_string();
    let child_id = format!("{doc_id}#section-1");
    let index = LinkGraphIndex {
        root: PathBuf::from("/workspace"),
        include_dirs: Vec::new(),
        excluded_dirs: Vec::new(),
        docs_by_id: HashMap::from([(
            doc_id.clone(),
            LinkGraphDocument {
                id: doc_id.clone(),
                id_lower: doc_id.clone(),
                stem: "a".to_string(),
                stem_lower: "a".to_string(),
                path: "notes/a.md".to_string().into(),
                path_lower: "notes/a.md".to_string(),
                title: "Doc A".to_string(),
                title_lower: "doc a".to_string(),
                tags: Vec::new(),
                tags_lower: Vec::new(),
                lead: String::new(),
                doc_type: None,
                word_count: 0,
                search_text: String::new(),
                search_text_lower: String::new(),
                saliency_base: crate::link_graph::saliency::DEFAULT_SALIENCY_BASE,
                decay_rate: crate::link_graph::saliency::DEFAULT_DECAY_RATE,
                created_ts: None,
                modified_ts: None,
            },
        )]),
        sections_by_doc: HashMap::new(),
        passages_by_id: HashMap::new(),
        attachments_by_doc: HashMap::new(),
        trees_by_doc: HashMap::from([(
            doc_id.clone(),
            vec![PageIndexNode {
                node_id: doc_id.clone().into(),
                parent_id: None,
                title: "Doc A".to_string(),
                level: 1,
                text: Arc::<str>::from(""),
                summary: None,
                children: vec![PageIndexNode {
                    node_id: child_id.clone().into(),
                    parent_id: Some(doc_id.clone()),
                    title: "Section".to_string(),
                    level: 2,
                    text: Arc::<str>::from(""),
                    summary: None,
                    children: Vec::new(),
                    metadata: PageIndexMeta {
                        line_range: (2, 3),
                        byte_range: None,
                        structural_path: vec!["Doc A".to_string(), "Section".to_string()],
                        content_hash: None,
                        attributes: HashMap::new(),
                        token_count: 0,
                        is_thinned: false,
                        logbook: Vec::new(),
                        observations: Vec::new(),
                    },
                    blocks: Vec::new(),
                }],
                metadata: PageIndexMeta {
                    line_range: (1, 3),
                    byte_range: None,
                    structural_path: vec!["Doc A".to_string()],
                    content_hash: None,
                    attributes: HashMap::new(),
                    token_count: 0,
                    is_thinned: false,
                    logbook: Vec::new(),
                    observations: Vec::new(),
                },
                blocks: Vec::new(),
            }],
        )]),
        node_parent_map: HashMap::from([(doc_id.clone(), None), (child_id.clone(), Some(doc_id))]),
        explicit_id_registry: HashMap::new(),
        alias_to_doc_id: HashMap::new(),
        outgoing: HashMap::new(),
        incoming: HashMap::new(),
        rank_by_id: HashMap::new(),
        edge_count: 0,
        virtual_nodes: HashMap::new(),
        symbol_to_docs: HashMap::new(),
    };

    assert_eq!(
        index.page_index_semantic_path(&child_id),
        Some(vec!["Doc A".to_string(), "Section".to_string()])
    );
    assert_eq!(
        index.page_index_trace_label(&child_id).as_deref(),
        Some("[Path: Doc A > Section]")
    );
}
