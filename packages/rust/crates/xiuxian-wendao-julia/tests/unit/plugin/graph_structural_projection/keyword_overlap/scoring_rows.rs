use super::{
    GraphStructuralKeywordOverlapPairInputs, GraphStructuralKeywordTagQueryInputs,
    GraphStructuralNodeMetadataInputs, GraphStructuralPairCandidateInputs, ResultTestExt,
    assert_f64_eq, build_graph_structural_keyword_overlap_pair_rerank_request_row,
    build_graph_structural_keyword_overlap_pair_rerank_request_row_from_metadata,
    build_graph_structural_keyword_tag_pair_rerank_request_row,
    build_graph_structural_keyword_tag_query_context,
    build_graph_structural_keyword_tag_rerank_signals, build_graph_structural_rerank_request_batch,
};
#[test]
fn build_graph_structural_keyword_tag_query_context_orders_keyword_before_tag() {
    let query = build_graph_structural_keyword_tag_query_context(
        "query-5",
        0,
        2,
        vec![" alpha ".to_string()],
        vec![" core ".to_string(), " graph ".to_string()],
        vec!["depends_on".to_string()],
    )
    .or_panic("query context should normalize");

    assert_eq!(query.query_id(), "query-5");
    assert_eq!(query.anchors()[0].plane(), "keyword");
    assert_eq!(query.anchors()[0].value(), "alpha");
    assert_eq!(query.anchors()[1].plane(), "tag");
    assert_eq!(query.anchors()[1].value(), "core");
    assert_eq!(query.anchors()[2].plane(), "tag");
    assert_eq!(query.anchors()[2].value(), "graph");
    assert_eq!(query.edge_constraint_kinds(), &["depends_on".to_string()]);
}

#[test]
fn build_graph_structural_keyword_tag_rerank_signals_maps_binary_matches() {
    let signals = build_graph_structural_keyword_tag_rerank_signals(0.6, 0.2, true, false)
        .or_panic("binary match signals should normalize");

    assert_f64_eq(signals.semantic_score(), 0.6);
    assert_f64_eq(signals.dependency_score(), 0.2);
    assert_f64_eq(signals.keyword_score(), 1.0);
    assert_f64_eq(signals.tag_score(), 0.0);
}

#[test]
fn build_graph_structural_keyword_tag_pair_rerank_request_row_composes_helper_layers() {
    let row = build_graph_structural_keyword_tag_pair_rerank_request_row(
        GraphStructuralKeywordTagQueryInputs::new(
            "query-7",
            1,
            3,
            vec!["alpha".to_string()],
            vec!["core".to_string()],
            vec!["depends_on".to_string()],
        ),
        GraphStructuralPairCandidateInputs::new(
            "node-b",
            "node-a",
            vec!["semantic_similar".to_string()],
        ),
        0.75,
        0.1,
        true,
        true,
    )
    .or_panic("combined helper should normalize");
    let batch = build_graph_structural_rerank_request_batch(std::slice::from_ref(&row))
        .or_panic("combined helper batch should validate");

    assert_eq!(row.query_id, "query-7");
    assert_eq!(row.candidate_id, "pair:node-a:node-b");
    assert_eq!(row.anchor_planes, vec!["keyword", "tag"]);
    assert_eq!(row.anchor_values, vec!["alpha", "core"]);
    assert_eq!(row.candidate_edge_kinds, vec!["semantic_similar"]);
    assert_f64_eq(row.keyword_score, 1.0);
    assert_f64_eq(row.tag_score, 1.0);
    assert_eq!(batch.num_rows(), 1);
}

#[test]
fn build_graph_structural_keyword_overlap_pair_rerank_request_row_computes_tag_overlap() {
    let row = build_graph_structural_keyword_overlap_pair_rerank_request_row(
        GraphStructuralKeywordTagQueryInputs::new(
            "query-8",
            0,
            2,
            vec!["alpha".to_string()],
            Vec::new(),
            Vec::new(),
        ),
        vec!["alpha".to_string(), "core".to_string()],
        vec!["graph".to_string(), "core".to_string()],
        GraphStructuralPairCandidateInputs::new(
            "node-z",
            "node-a",
            vec!["semantic_similar".to_string()],
        ),
        0.8,
        0.0,
        true,
    )
    .or_panic("tag-overlap pair helper should normalize");
    let batch = build_graph_structural_rerank_request_batch(std::slice::from_ref(&row))
        .or_panic("tag-overlap batch should validate");

    assert_eq!(row.candidate_id, "pair:node-a:node-z");
    assert_eq!(row.anchor_planes, vec!["keyword", "tag"]);
    assert_eq!(row.anchor_values, vec!["alpha", "core"]);
    assert_f64_eq(row.keyword_score, 1.0);
    assert_f64_eq(row.tag_score, 1.0);
    assert_eq!(batch.num_rows(), 1);
}

#[test]
fn build_graph_structural_keyword_overlap_pair_rerank_request_row_from_metadata_composes() {
    let row = build_graph_structural_keyword_overlap_pair_rerank_request_row_from_metadata(
        GraphStructuralKeywordOverlapPairInputs::new(
            GraphStructuralKeywordTagQueryInputs::new(
                "query-9",
                0,
                2,
                vec!["alpha".to_string()],
                Vec::new(),
                vec!["semantic_similar".to_string()],
            ),
            GraphStructuralNodeMetadataInputs::new(vec!["alpha".to_string(), "core".to_string()]),
            GraphStructuralNodeMetadataInputs::new(vec!["graph".to_string(), "core".to_string()]),
            GraphStructuralPairCandidateInputs::new(
                "node-k",
                "node-a",
                vec!["semantic_similar".to_string()],
            ),
        ),
        0.9,
        0.1,
        true,
    )
    .or_panic("metadata-aware overlap helper should normalize");
    let batch = build_graph_structural_rerank_request_batch(std::slice::from_ref(&row))
        .or_panic("metadata-aware overlap batch should validate");

    assert_eq!(row.query_id, "query-9");
    assert_eq!(row.candidate_id, "pair:node-a:node-k");
    assert_eq!(row.anchor_planes, vec!["keyword", "tag"]);
    assert_eq!(row.anchor_values, vec!["alpha", "core"]);
    assert_eq!(row.edge_constraint_kinds, vec!["semantic_similar"]);
    assert_f64_eq(row.keyword_score, 1.0);
    assert_f64_eq(row.tag_score, 1.0);
    assert_eq!(batch.num_rows(), 1);
}
