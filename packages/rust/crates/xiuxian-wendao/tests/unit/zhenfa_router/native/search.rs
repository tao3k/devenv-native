use super::{
    LinkGraphPlannedSearchPayload, LinkGraphSearchOptions, WendaoSearchArgs, normalize_limit,
    render_xml_lite_hits, validate_root_dir_argument,
};
use crate::link_graph::LinkGraphHit;
use crate::link_graph::{LinkGraphConfidenceLevel, LinkGraphRetrievalMode};

#[test]
fn wendao_search_args_deserialize_query_vector() {
    let args: WendaoSearchArgs = serde_json::from_value(serde_json::json!({
        "query": "native zhenfa",
        "query_vector": [1.0, 0.0, 0.0]
    }))
    .unwrap_or_else(|error| panic!("deserialize native args: {error}"));

    assert_eq!(args.query, "native zhenfa");
    assert_eq!(args.query_vector, Some(vec![1.0, 0.0, 0.0]));
}

#[test]
fn normalize_limit_defaults_and_clamps() {
    assert_eq!(normalize_limit(None), 20);
    assert_eq!(normalize_limit(Some(0)), 1);
    assert_eq!(normalize_limit(Some(42)), 42);
    assert_eq!(normalize_limit(Some(999)), 200);
}

#[test]
fn validate_root_dir_argument_accepts_real_paths() {
    assert!(validate_root_dir_argument(None).is_ok());
    assert!(validate_root_dir_argument(Some("docs")).is_ok());
    assert!(validate_root_dir_argument(Some("  docs ")).is_ok());
}

#[test]
fn validate_root_dir_argument_rejects_blank_values() {
    assert!(validate_root_dir_argument(Some("")).is_err());
    assert!(validate_root_dir_argument(Some("   ")).is_err());
}

#[test]
fn render_xml_lite_prefers_path_and_semantic_hit_type() {
    let payload = LinkGraphPlannedSearchPayload {
        query: "journal".to_string(),
        options: LinkGraphSearchOptions::default(),
        hits: Vec::new(),
        hit_count: 1,
        section_hit_count: 0,
        requested_mode: LinkGraphRetrievalMode::default(),
        selected_mode: LinkGraphRetrievalMode::default(),
        reason: String::new(),
        graph_hit_count: 1,
        source_hint_count: 0,
        graph_confidence_score: 0.0,
        graph_confidence_level: LinkGraphConfidenceLevel::default(),
        retrieval_plan: None,
        semantic_ignition: None,
        julia_rerank: None,
        query_vector: None,
        quantum_contexts: Vec::new(),
        results: vec![LinkGraphHit {
            stem: "daily".to_string(),
            title: "Daily Journal".to_string(),
            path: "journal/daily.md".to_string().into(),
            doc_type: None,
            tags: Vec::new(),
            score: 0.9,
            best_section: None,
            match_reason: None,
        }],
        provisional_suggestions: Vec::new(),
        provisional_error: None,
        promoted_overlay: None,
        ccs_audit: None,
    };

    let rendered = render_xml_lite_hits(&payload);
    assert!(rendered.contains("<hit id=\"journal/daily.md\""));
    assert!(rendered.contains("type=\"journal\""));
}

#[test]
fn render_xml_lite_prefers_frontmatter_doc_type_over_tags_and_path() {
    let payload = LinkGraphPlannedSearchPayload {
        query: "agenda".to_string(),
        options: LinkGraphSearchOptions::default(),
        hits: Vec::new(),
        hit_count: 1,
        section_hit_count: 0,
        requested_mode: LinkGraphRetrievalMode::default(),
        selected_mode: LinkGraphRetrievalMode::default(),
        reason: String::new(),
        graph_hit_count: 1,
        source_hint_count: 0,
        graph_confidence_score: 0.0,
        graph_confidence_level: LinkGraphConfidenceLevel::default(),
        retrieval_plan: None,
        semantic_ignition: None,
        julia_rerank: None,
        query_vector: None,
        quantum_contexts: Vec::new(),
        results: vec![LinkGraphHit {
            stem: "override".to_string(),
            title: "Override".to_string(),
            path: "journal/override.md".to_string().into(),
            doc_type: Some("agenda".to_string()),
            tags: vec!["journal".to_string()],
            score: 0.9,
            best_section: None,
            match_reason: None,
        }],
        provisional_suggestions: Vec::new(),
        provisional_error: None,
        promoted_overlay: None,
        ccs_audit: None,
    };

    let rendered = render_xml_lite_hits(&payload);
    assert!(rendered.contains("<hit id=\"journal/override.md\""));
    assert!(rendered.contains("type=\"agenda\""));
}
