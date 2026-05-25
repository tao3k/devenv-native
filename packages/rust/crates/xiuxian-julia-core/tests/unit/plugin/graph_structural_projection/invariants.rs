use super::{
    GraphStructuralCandidateSubgraph, GraphStructuralKeywordOverlapCandidateInputs,
    GraphStructuralNodeMetadataInputs, GraphStructuralPairCandidateInputs,
    GraphStructuralQueryContext, GraphStructuralRerankSignals,
    build_graph_structural_keyword_overlap_candidate_inputs,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_tag_query_context,
    build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples,
    graph_structural_pair_candidate_id, graph_structural_shared_tag_anchors,
};
use crate::julia_plugin_test_support::common::ResultTestExt;

#[test]
fn build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples_rejects_blank_endpoint()
 {
    let error =
        build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
            "candidate-from-tuples",
            vec![("node-1", "", 0.4)],
            "related",
            0.3,
            1.0,
            0.0,
        )
        .err_or_panic("blank endpoint should fail");

    assert!(
        error
            .to_string()
            .contains("pair right id must not be blank"),
        "unexpected error: {error}"
    );
}

#[test]
fn build_graph_structural_keyword_tag_query_context_rejects_empty_anchor_lists() {
    let error = build_graph_structural_keyword_tag_query_context(
        "query-6",
        0,
        1,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .err_or_panic("query context should reject empty keyword and tag anchors");
    assert!(
        error
            .to_string()
            .contains("at least one query anchor is required"),
        "unexpected error: {error}"
    );
}

#[test]
fn graph_structural_shared_tag_anchors_preserve_left_order_and_uniqueness() {
    let shared = graph_structural_shared_tag_anchors(
        vec![
            "core".to_string(),
            "alpha".to_string(),
            "core".to_string(),
            "graph".to_string(),
        ],
        vec!["graph".to_string(), "core".to_string(), "delta".to_string()],
    )
    .or_panic("shared tag anchors should normalize");

    assert_eq!(shared, vec!["core".to_string(), "graph".to_string()]);
}

#[test]
fn graph_structural_keyword_overlap_candidate_inputs_new_composes() {
    let candidate = build_graph_structural_keyword_overlap_candidate_inputs(
        build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
            "node-left",
            "node-right",
            vec!["semantic_similar".to_string()],
            vec!["alpha".to_string(), "core".to_string()],
            vec!["core".to_string()],
        ),
        0.6,
        0.2,
        true,
    );

    assert_eq!(
        candidate,
        GraphStructuralKeywordOverlapCandidateInputs::new(
            GraphStructuralNodeMetadataInputs::new(vec!["alpha".to_string(), "core".to_string(),]),
            GraphStructuralNodeMetadataInputs::new(vec!["core".to_string()]),
            GraphStructuralPairCandidateInputs::new(
                "node-left",
                "node-right",
                vec!["semantic_similar".to_string()],
            ),
            0.6,
            0.2,
            true,
        )
    );
}

#[test]
fn graph_structural_query_context_rejects_empty_anchor_list() {
    let error = GraphStructuralQueryContext::new("query-1", 0, 1, Vec::new(), Vec::new())
        .err_or_panic("query context should reject empty anchors");
    assert!(
        error
            .to_string()
            .contains("at least one query anchor is required"),
        "unexpected error: {error}"
    );
}

#[test]
fn graph_structural_candidate_subgraph_rejects_blank_node_ids() {
    let error = GraphStructuralCandidateSubgraph::new(
        "candidate-a",
        vec!["node-a".to_string(), "  ".to_string()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .err_or_panic("candidate should reject blank node ids");
    assert!(
        error
            .to_string()
            .contains("candidate node ids item 1 must not be blank"),
        "unexpected error: {error}"
    );
}

#[test]
fn graph_structural_rerank_signals_reject_negative_scores() {
    let error = GraphStructuralRerankSignals::new(-0.1, 0.0, 0.0, 0.0)
        .err_or_panic("signals should reject negative scores");
    assert!(
        error
            .to_string()
            .contains("semantic score must be non-negative"),
        "unexpected error: {error}"
    );
}

#[test]
fn graph_structural_pair_candidate_id_rejects_duplicate_endpoints() {
    let error = graph_structural_pair_candidate_id("same-node", "same-node")
        .err_or_panic("pair candidate id should reject duplicate endpoints");
    assert!(
        error
            .to_string()
            .contains("pair endpoints must not resolve to the same id"),
        "unexpected error: {error}"
    );
}
